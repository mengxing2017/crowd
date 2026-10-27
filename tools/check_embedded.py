"""POSIX lifecycle smoke test for the real embedded-PG development entry."""

import json
import os
from pathlib import Path
import signal
import socket
import subprocess
import tempfile
import time
import urllib.request
import stat


REPO = Path(__file__).resolve().parents[1]
BINARY = REPO / "backend/target/debug/examples/embedded_postgres"
HTTP = urllib.request.build_opener(urllib.request.ProxyHandler({}))


def port_is_open(port):
    with socket.socket() as connection:
        connection.settimeout(1)
        return connection.connect_ex(("127.0.0.1", port)) == 0


def assert_database_cleaned(log):
    started = []
    for line in log.splitlines():
        try:
            fields = json.loads(line).get("fields", {})
        except ValueError:
            continue
        if fields.get("message") == "embedded PostgreSQL started; data will be removed on exit":
            started.append(fields)
    assert len(started) == 1, f"missing database startup evidence:\n{log}"
    database = started[0]
    assert not Path(database["data_directory"]).exists(), "temporary data was not removed"
    assert not port_is_open(database["database_port"]), "embedded database still accepts connections"


def check_case(shutdown_signal=None):
    # Keep the socket occupied for the failure case; release it for normal runs.
    with socket.socket() as reserved:
        reserved.bind(("127.0.0.1", 0))
        port = reserved.getsockname()[1]
        reserved.listen()
        if shutdown_signal is not None:
            reserved.close()
        env = {
            **os.environ,
            "CROWD_BIND_ADDR": f"127.0.0.1:{port}",
            "RUST_LOG": "crowd_api=info,embedded_postgres=info",
            # The example must ignore all external database connection settings.
            "CROWD_DATABASE_URL": "postgres://unused-test-value@invalid.example/never_connect",
            "CROWD_TEST_DATABASE_URL": "postgres://unused-test-value@invalid.example/crowd_test_external",
            # Production bootstrap configuration must not affect temporary development accounts.
            "CROWD_BOOTSTRAP_ADMIN_USERNAME": "invalid production username",
            "CROWD_BOOTSTRAP_ADMIN_PASSWORD": "unused-production-secret",
        }
        env.pop("CROWD_DEV_ADMIN_PASSWORD", None)
        if shutdown_signal == signal.SIGTERM:
            env["CROWD_DEV_ADMIN_PASSWORD"] = "custom-local-admin-password"
        with tempfile.TemporaryFile(mode="w+") as output:
            proc = subprocess.Popen([str(BINARY)], cwd=REPO, env=env, stdout=output, stderr=subprocess.STDOUT)
            try:
                if shutdown_signal is None:
                    assert proc.wait(timeout=120) != 0, "occupied HTTP port should fail startup"
                else:
                    deadline = time.monotonic() + 120
                    while time.monotonic() < deadline:
                        if proc.poll() is not None:
                            output.seek(0)
                            raise AssertionError(f"development entry exited early:\n{output.read()}")
                        try:
                            with HTTP.open(f"http://127.0.0.1:{port}/readyz", timeout=1) as response:
                                if json.load(response) == {"status": "ready"}:
                                    break
                        except OSError:
                            pass
                        time.sleep(0.2)
                    else:
                        raise AssertionError("embedded API did not become ready")
                    with HTTP.open(f"http://127.0.0.1:{port}/api/v1/system", timeout=2) as response:
                        assert json.load(response)["data"]["attributes"]["application"] == "crowd"
                    output.seek(0)
                    events = [json.loads(line).get("fields", {}) for line in output.read().splitlines() if line.startswith("{")]
                    credentials = Path(next(event["credential_file"] for event in events if "credential_file" in event))
                    assert stat.S_IMODE(credentials.stat().st_mode) == 0o600
                    values = dict(line.split("=", 1) for line in credentials.read_text().splitlines())
                    body = json.dumps({"username": values["username"], "password": values["password"]}).encode()
                    request = urllib.request.Request(f"http://127.0.0.1:{port}/api/v1/community/admin/login", data=body, headers={"Content-Type": "application/json"})
                    with HTTP.open(request, timeout=10) as response:
                        token = json.load(response)["data"]["token"]
                    request = urllib.request.Request(f"http://127.0.0.1:{port}/api/v1/community/read/rules", headers={"Authorization": f"Bearer {token}"})
                    with HTTP.open(request, timeout=3) as response:
                        assert json.load(response)["data"] == []
                    output.seek(0)
                    startup_output = output.read()
                    assert "开发管理员用户名：admin" in startup_output
                    if "CROWD_DEV_ADMIN_PASSWORD" in env:
                        assert values["password"] == env["CROWD_DEV_ADMIN_PASSWORD"]
                        assert "管理员密码：使用 CROWD_DEV_ADMIN_PASSWORD 设置的值" in startup_output
                        assert values["password"] not in startup_output
                    else:
                        assert f"本次临时管理员密码：{values['password']}" in startup_output
                    assert all(values["password"] not in json.dumps(event) for event in events), "password entered structured application logs"
                    proc.send_signal(shutdown_signal)
                    assert proc.wait(timeout=20) == 0, "shutdown was not graceful"
                output.seek(0)
                log = output.read()
                assert "unused-test-value" not in log, "external connection credential leaked"
                assert "unused-production-secret" not in log, "production bootstrap credential leaked"
                assert_database_cleaned(log)
            finally:
                if proc.poll() is None:
                    proc.send_signal(signal.SIGINT)
                    try:
                        proc.wait(timeout=20)
                    except subprocess.TimeoutExpired:
                        proc.kill()
                        proc.wait()


if __name__ == "__main__":
    if os.name != "posix":
        raise SystemExit("This signal lifecycle check requires POSIX; use Ctrl+C manually on Windows.")
    for stop_signal in (signal.SIGINT, signal.SIGTERM):
        check_case(stop_signal)
        print(f"{stop_signal.name}: migration/readiness, isolated config, database stop/data cleanup passed")
    check_case()
    print("HTTP port conflict: error exit and database stop/data cleanup passed")

    env = {**os.environ, "CROWD_BIND_ADDR": "0.0.0.0:8080"}
    result = subprocess.run([str(BINARY)], cwd=REPO, env=env, capture_output=True, text=True, timeout=10)
    assert result.returncode != 0, "non-loopback binding must be rejected"
    assert "embedded PostgreSQL started" not in result.stdout
    print("Non-loopback HTTP binding: rejected before database setup")
