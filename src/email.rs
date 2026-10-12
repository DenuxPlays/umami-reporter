use crate::{
    config::{Config, SmtpSecurity},
    report::Report,
};
use anyhow::{Context, Result};
use lettre::{
    Message, SmtpTransport, Transport,
    message::{Mailbox, MultiPart, SinglePart, header::ContentType},
    transport::smtp::authentication::Credentials,
};
use minijinja::{AutoEscape, Environment, context};

pub struct RenderedEmail {
    pub subject: String,
    pub text: String,
    pub html: String,
}

pub fn render(report: &Report) -> Result<RenderedEmail> {
    let mut environment = Environment::new();
    environment.set_auto_escape_callback(|name| {
        if name.ends_with(".html") {
            AutoEscape::Html
        } else {
            AutoEscape::None
        }
    });
    environment.add_template("report.txt", include_str!("../templates/report.txt"))?;
    environment.add_template("report.html", include_str!("../templates/report.html"))?;

    let template_context = context! {
        month => report.month.clone(),
        visitors => report.visitors,
        visits => report.visits,
        views => report.views,
    };
    Ok(RenderedEmail {
        subject: format!("Umami-Monatsbericht für {}", report.month),
        text: environment
            .get_template("report.txt")?
            .render(&template_context)?,
        html: environment
            .get_template("report.html")?
            .render(&template_context)?,
    })
}

pub fn send(config: &Config, recipients: &[Mailbox], rendered: RenderedEmail) -> Result<()> {
    let transport_builder = match config.smtp_security {
        SmtpSecurity::Starttls => SmtpTransport::starttls_relay(&config.smtp_host),
        SmtpSecurity::Tls => SmtpTransport::relay(&config.smtp_host),
        SmtpSecurity::None => Ok(SmtpTransport::builder_dangerous(&config.smtp_host)),
    }
    .context("SMTP-Verbindung konnte nicht erstellt werden")?;
    let transport = transport_builder
        .port(config.smtp_port)
        .credentials(Credentials::new(
            config.smtp_username.clone(),
            config.smtp_password.clone(),
        ))
        .build();

    for recipient in recipients {
        let email = build_message(config, recipient, &rendered)?;
        transport
            .send(&email)
            .with_context(|| format!("E-Mail an {recipient} konnte nicht versendet werden"))?;
    }
    Ok(())
}

fn build_message(
    config: &Config,
    recipient: &Mailbox,
    rendered: &RenderedEmail,
) -> Result<Message> {
    Message::builder()
        .from(config.smtp_from.clone())
        .to(recipient.clone())
        .subject(&rendered.subject)
        .multipart(
            MultiPart::alternative()
                .singlepart(
                    SinglePart::builder()
                        .header(ContentType::TEXT_PLAIN)
                        .body(rendered.text.clone()),
                )
                .singlepart(
                    SinglePart::builder()
                        .header(ContentType::TEXT_HTML)
                        .body(rendered.html.clone()),
                ),
        )
        .context("E-Mail konnte nicht erstellt werden")
}

#[cfg(test)]
mod tests {
    use super::{RenderedEmail, build_message, render};
    use crate::config::{Config, SmtpSecurity};
    use crate::report::Report;
    use lettre::message::Mailbox;

    #[test]
    fn renders_german_plain_text_and_html() {
        let report = Report {
            month: "09.2026".into(),
            visitors: 12,
            visits: 34,
            views: 56,
        };
        let email = render(&report).unwrap();
        assert!(email.text.contains("Besucher: 12"));
        assert!(email.text.contains("Besuche: 34"));
        assert!(email.text.contains("Seitenaufrufe: 56"));
        assert!(email.html.contains("<html lang=\"de\">"));
        assert!(email.html.contains("Seitenaufrufe"));
        assert!(email.html.contains(">56</p>"));
    }

    #[test]
    fn builds_one_message_with_only_its_recipient() {
        let config = Config {
            umami_url: "https://example.org".into(),
            umami_website_id: "website".into(),
            umami_api_key: "key".into(),
            smtp_host: "smtp.example.org".into(),
            smtp_port: 587,
            smtp_security: SmtpSecurity::Starttls,
            smtp_username: "user".into(),
            smtp_password: "password".into(),
            smtp_from: "sender@example.org".parse().unwrap(),
        };
        let rendered = RenderedEmail {
            subject: "Bericht".into(),
            text: "Text".into(),
            html: "<p>HTML</p>".into(),
        };
        let recipient: Mailbox = "recipient@example.org".parse().unwrap();
        let message = build_message(&config, &recipient, &rendered).unwrap();

        assert_eq!(message.envelope().to().len(), 1);
        assert_eq!(
            message.envelope().to()[0].to_string(),
            "recipient@example.org"
        );
    }
}
