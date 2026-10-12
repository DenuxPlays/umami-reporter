use anyhow::{Context, Result, bail};
use lettre::message::Mailbox;
use std::{env, str::FromStr};

#[derive(Clone)]
pub struct Config {
    pub umami_url: String,
    pub umami_website_id: String,
    pub umami_api_key: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_security: SmtpSecurity,
    pub smtp_username: String,
    pub smtp_password: String,
    pub smtp_from: Mailbox,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SmtpSecurity {
    Starttls,
    Tls,
    None,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            umami_url: required_env("UMAMI_URL")?,
            umami_website_id: required_env("UMAMI_WEBSITE_ID")?,
            umami_api_key: required_env("UMAMI_API_KEY")?,
            smtp_host: required_env("SMTP_HOST")?,
            smtp_port: env::var("SMTP_PORT")
                .unwrap_or_else(|_| "587".into())
                .parse()
                .context("SMTP_PORT muss eine Zahl sein")?,
            smtp_security: env::var("SMTP_SECURITY")
                .unwrap_or_else(|_| "starttls".into())
                .parse()?,
            smtp_username: required_env("SMTP_USERNAME")?,
            smtp_password: required_env("SMTP_PASSWORD")?,
            smtp_from: required_env("SMTP_FROM")?
                .parse()
                .context("SMTP_FROM ist keine gültige E-Mail-Adresse")?,
        })
    }
}

impl FromStr for SmtpSecurity {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "starttls" => Ok(Self::Starttls),
            "tls" | "smtps" => Ok(Self::Tls),
            "none" | "unencrypted" => Ok(Self::None),
            _ => bail!("SMTP_SECURITY muss starttls, tls oder none sein (war: {value})"),
        }
    }
}

fn required_env(name: &str) -> Result<String> {
    env::var(name).with_context(|| format!("Umgebungsvariable {name} fehlt"))
}

#[cfg(test)]
mod tests {
    use super::SmtpSecurity;
    use std::str::FromStr;

    #[test]
    fn parses_supported_security_modes() {
        assert_eq!(
            SmtpSecurity::from_str("STARTTLS").unwrap(),
            SmtpSecurity::Starttls
        );
        assert_eq!(SmtpSecurity::from_str("smtps").unwrap(), SmtpSecurity::Tls);
        assert_eq!(
            SmtpSecurity::from_str("unencrypted").unwrap(),
            SmtpSecurity::None
        );
    }

    #[test]
    fn rejects_unknown_security_mode() {
        assert!(SmtpSecurity::from_str("insecure-ish").is_err());
    }
}
