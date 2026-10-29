//! External database configuration; credentials are passed directly to SQLx.
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use std::io::{Error, ErrorKind};

pub fn from_env() -> Result<PgConnectOptions, Error> {
    from_lookup(|key| std::env::var(key).ok())
}

fn invalid(message: impl Into<String>) -> Error {
    Error::new(ErrorKind::InvalidInput, message.into())
}

fn from_lookup(get: impl Fn(&str) -> Option<String>) -> Result<PgConnectOptions, Error> {
    if let Some(url) = get("CROWD_DATABASE_URL").filter(|value| !value.is_empty()) {
        return url
            .parse()
            .map_err(|_| invalid("invalid CROWD_DATABASE_URL"));
    }
    let required = |key: &str| {
        get(key).filter(|value| !value.is_empty()).ok_or_else(|| {
            invalid(format!(
                "missing {key}; set database fields or CROWD_DATABASE_URL"
            ))
        })
    };
    let port = get("CROWD_DATABASE_PORT")
        .unwrap_or_else(|| "5432".into())
        .parse::<u16>()
        .ok()
        .filter(|port| *port != 0)
        .ok_or_else(|| invalid("invalid CROWD_DATABASE_PORT; expected 1-65535"))?;
    let ssl_mode = get("CROWD_DATABASE_SSL_MODE")
        .unwrap_or_else(|| "prefer".into())
        .parse::<PgSslMode>()
        .map_err(|_| invalid("invalid CROWD_DATABASE_SSL_MODE"))?;
    Ok(PgConnectOptions::new()
        .host(&required("CROWD_DATABASE_HOST")?)
        .port(port)
        .database(&required("CROWD_DATABASE_NAME")?)
        .username(&required("CROWD_DATABASE_USER")?)
        .password(&required("CROWD_DATABASE_PASSWORD")?)
        .ssl_mode(ssl_mode))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::ConnectOptions;

    fn config(overrides: &[(&str, &str)]) -> Result<PgConnectOptions, Error> {
        let mut values = std::collections::HashMap::from([
            ("CROWD_DATABASE_HOST", "db"),
            ("CROWD_DATABASE_NAME", "crowd"),
            ("CROWD_DATABASE_USER", "crowd"),
            ("CROWD_DATABASE_PASSWORD", "p@ss:/?#% word"),
        ]);
        values.extend(overrides.iter().copied());
        from_lookup(|key| values.get(key).map(|value| (*value).to_owned()))
    }

    #[test]
    fn fields_preserve_special_password_characters() {
        let options = config(&[("CROWD_DATABASE_PORT", "5433")]).unwrap();
        assert_eq!(options.get_host(), "db");
        assert_eq!(options.get_port(), 5433);
        assert_eq!(options.get_database(), Some("crowd"));
        assert_eq!(options.get_username(), "crowd");
        assert!(matches!(options.get_ssl_mode(), PgSslMode::Prefer));
        // SQLx serializes the raw password as a URL component only on request.
        assert_eq!(
            options.to_url_lossy().password(),
            Some("p%40ss%3A%2F%3F%23%25%20word")
        );
    }

    #[test]
    fn url_takes_precedence_and_empty_url_uses_fields() {
        let options = config(&[
            (
                "CROWD_DATABASE_URL",
                "postgres://old:encoded%40password@legacy:5434/existing?sslmode=require",
            ),
            ("CROWD_DATABASE_PORT", "invalid"),
        ])
        .unwrap();
        assert_eq!(options.get_host(), "legacy");
        assert_eq!(options.get_port(), 5434);
        assert!(matches!(options.get_ssl_mode(), PgSslMode::Require));
        assert_eq!(
            options.to_url_lossy().password(),
            Some("encoded%40password")
        );
        assert_eq!(
            config(&[("CROWD_DATABASE_URL", "")]).unwrap().get_host(),
            "db"
        );
    }

    #[test]
    fn configuration_errors_do_not_expose_credentials() {
        for overrides in [
            vec![("CROWD_DATABASE_URL", "invalid-secret-url")],
            vec![("CROWD_DATABASE_PORT", "secret-invalid-port")],
            vec![("CROWD_DATABASE_PORT", "0")],
            vec![("CROWD_DATABASE_SSL_MODE", "secret-invalid-mode")],
            vec![("CROWD_DATABASE_PASSWORD", "")],
            vec![("CROWD_DATABASE_HOST", "")],
        ] {
            let error = config(&overrides).unwrap_err().to_string();
            assert!(!error.contains("secret"));
            assert!(!error.contains("p@ss"));
        }
        assert!(from_lookup(|_| None).is_err());
        assert!(matches!(
            config(&[("CROWD_DATABASE_SSL_MODE", "disable")])
                .unwrap()
                .get_ssl_mode(),
            PgSslMode::Disable
        ));
    }
}
