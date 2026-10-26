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
        }
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
                    proc.send_signal(shutdown_signal)
                    assert proc.wait(timeout=20) == 0, "shutdown was not graceful"
                output.seek(0)
                log = output.read()
                assert "unused-test-value" not in log, "external connection credential leaked"
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
