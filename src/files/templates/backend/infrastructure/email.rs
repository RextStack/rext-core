//! Email service
//!
//! Module to handle sending emails. Configurable via .env file.
//! Currently supports SMTP and templates made in handlers, not file-based templates.
//!
//! Example usage:
//! ```rust_no_run
//! // Initialize the service
//! let email_service = EmailService::from_env()?;
//!
//! // Send a welcome email (1-line function call!)
//! let result = email_service.send_welcome_email(
//!     "example_to_email@gmail.com",
//!     "Example User",
//!     "Example App Name"
//! ).await;
//!
//! println!("Email result: {:?}", result);
//! ```

use lettre::message::Mailbox;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fmt::Display;
use std::str::FromStr;
use tracing::{error, info};

/// Represents all supported email services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmailServiceType {
    SMTP,
}

impl Display for EmailServiceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for EmailServiceType {
    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        Ok(Self::SMTP)
    }

    type Err = String;
}

/// Email service configuration
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct EmailConfig {
    /// SMTP service provider (currently only "smtp" is supported)
    pub service_type: EmailServiceType,
    /// SMTP server hostname
    pub smtp_host: String,
    /// SMTP server port
    pub smtp_port: u16,
    /// SMTP username
    pub smtp_username: String,
    /// SMTP password
    pub smtp_password: String,
    /// From email address
    pub from_email: String,
    /// From name (display name)
    pub from_name: String,
    /// Reply-to email address (optional)
    pub reply_to_email: Option<String>,
    /// Reply-to name (optional)
    pub reply_to_name: Option<String>,
}

/// Email template for sending
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailTemplate {
    pub subject: String,
    pub body: String,
    pub content_type: EmailContentType,
}

/// Content type for emails
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmailContentType {
    Text,
    Html,
}

/// Email sending result
#[derive(Debug)]
#[allow(dead_code)]
pub enum EmailResult {
    Success,
    Failed(String),
}

/// Main email service struct
#[allow(dead_code)]
pub struct EmailService {
    config: EmailConfig,
    transport: SmtpTransport,
}

impl EmailService {
    /// Initialize the email service from environment variables
    #[allow(dead_code)]
    pub fn from_env() -> Result<Self, String> {
        let config = EmailConfig::from_env()?;
        let transport = Self::create_transport(&config)?;

        Ok(Self { config, transport })
    }

    /// Create a new email service with custom configuration
    #[allow(dead_code)]
    pub fn new(config: EmailConfig) -> Result<Self, String> {
        let transport = Self::create_transport(&config)?;

        Ok(Self { config, transport })
    }

    /// Create SMTP transport based on configuration
    #[allow(dead_code)]
    fn create_transport(config: &EmailConfig) -> Result<SmtpTransport, String> {
        if config.service_type.to_string().to_lowercase() != "smtp" {
            return Err(format!(
                "Unsupported email service type: {}",
                config.service_type
            ));
        }

        let credentials =
            Credentials::new(config.smtp_username.clone(), config.smtp_password.clone());

        // Configure transport with proper TLS settings
        let transport = if config.smtp_port == 465 {
            // Port 465 uses implicit TLS (SSL)
            SmtpTransport::relay(&config.smtp_host)
                .map_err(|e| format!("Failed to create SMTP relay: {}", e))?
                .port(config.smtp_port)
                .credentials(credentials)
                .build()
        } else {
            // Port 587 and others use STARTTLS
            SmtpTransport::starttls_relay(&config.smtp_host)
                .map_err(|e| format!("Failed to create SMTP STARTTLS relay: {}", e))?
                .port(config.smtp_port)
                .credentials(credentials)
                .build()
        };

        Ok(transport)
    }

    /// Send a templated email to a single recipient
    #[allow(dead_code)]
    pub async fn send_template_email(
        &self,
        to_email: &str,
        to_name: Option<&str>,
        template_name: &str,
        variables: Option<HashMap<String, String>>,
    ) -> EmailResult {
        let template = match Self::get_email_template(template_name) {
            Ok(template) => template,
            Err(e) => {
                error!("Failed to load email template '{}': {}", template_name, e);
                return EmailResult::Failed(format!("Template error: {}", e));
            }
        };

        self.send_email(to_email, to_name, &template, variables)
            .await
    }

    /// Send an email with a custom template
    #[allow(dead_code)]
    pub async fn send_email(
        &self,
        to_email: &str,
        to_name: Option<&str>,
        template: &EmailTemplate,
        variables: Option<HashMap<String, String>>,
    ) -> EmailResult {
        // Process template variables
        let processed_subject = Self::process_template_variables(&template.subject, &variables);
        let processed_body = Self::process_template_variables(&template.body, &variables);

        // Build the email message
        let message_result = self.build_message(
            to_email,
            to_name,
            &processed_subject,
            &processed_body,
            &template.content_type,
        );

        let message = match message_result {
            Ok(msg) => msg,
            Err(e) => {
                error!("Failed to build email message: {}", e);
                return EmailResult::Failed(format!("Message build error: {}", e));
            }
        };

        // Send the email
        match self.transport.send(&message) {
            Ok(_) => {
                info!("Email sent successfully to: {}", to_email);
                EmailResult::Success
            }
            Err(e) => {
                error!("Failed to send email to {}: {}", to_email, e);
                EmailResult::Failed(format!("SMTP error: {}", e))
            }
        }
    }

    /// Build the email message
    #[allow(dead_code)]
    fn build_message(
        &self,
        to_email: &str,
        to_name: Option<&str>,
        subject: &str,
        body: &str,
        content_type: &EmailContentType,
    ) -> Result<Message, String> {
        // Parse email addresses
        let from_mailbox = Mailbox::new(
            Some(self.config.from_name.clone()),
            self.config
                .from_email
                .parse()
                .map_err(|e| format!("Invalid from email address: {}", e))?,
        );

        let to_mailbox = Mailbox::new(
            to_name.map(|s| s.to_owned()),
            to_email
                .parse()
                .map_err(|e| format!("Invalid to email address: {}", e))?,
        );

        // Build the message
        let mut builder = Message::builder()
            .from(from_mailbox)
            .to(to_mailbox)
            .subject(subject);

        // Add reply-to if configured
        if let (Some(reply_email), Some(reply_name)) =
            (&self.config.reply_to_email, &self.config.reply_to_name)
        {
            let reply_mailbox = Mailbox::new(
                Some(reply_name.clone()),
                reply_email
                    .parse()
                    .map_err(|e| format!("Invalid reply-to email address: {}", e))?,
            );
            builder = builder.reply_to(reply_mailbox);
        } else if let Some(reply_email) = &self.config.reply_to_email {
            let reply_mailbox = Mailbox::new(
                None,
                reply_email
                    .parse()
                    .map_err(|e| format!("Invalid reply-to email address: {}", e))?,
            );
            builder = builder.reply_to(reply_mailbox);
        }

        // Set content type and body
        let message = match content_type {
            EmailContentType::Text => builder
                .header(ContentType::TEXT_PLAIN)
                .body(body.to_string()),
            EmailContentType::Html => builder
                .header(ContentType::TEXT_HTML)
                .body(body.to_string()),
        };

        message.map_err(|e| format!("Failed to build message: {}", e))
    }

    /// Process template variables in content
    #[allow(dead_code)]
    fn process_template_variables(
        content: &str,
        variables: &Option<HashMap<String, String>>,
    ) -> String {
        if let Some(vars) = variables {
            let mut processed = content.to_string();
            for (key, value) in vars {
                let placeholder = format!("{{{{{}}}}}", key);
                processed = processed.replace(&placeholder, value);
            }
            processed
        } else {
            content.to_string()
        }
    }

    /// Get a predefined email template by name
    #[allow(dead_code)]
    fn get_email_template(template_name: &str) -> Result<EmailTemplate, String> {
        match template_name {
            "welcome" => Ok(EmailTemplate {
                subject: "Welcome to {{app_name}}!".to_string(),
                body: "Hello {{user_name}},\n\nWelcome to {{app_name}}! We're excited to have you on board.\n\nBest regards,\nThe {{app_name}} Team".to_string(),
                content_type: EmailContentType::Text,
            }),
            "password_reset" => Ok(EmailTemplate {
                subject: "Password Reset Request".to_string(),
                body: "Hello {{user_name}},\n\nYou have requested a password reset. Click the link below to reset your password:\n\n{{reset_link}}\n\nIf you didn't request this, please ignore this email.\n\nBest regards,\nThe {{app_name}} Team".to_string(),
                content_type: EmailContentType::Text,
            }),
            "verification" => Ok(EmailTemplate {
                subject: "Please verify your email address".to_string(),
                body: "Hello {{user_name}},\n\nPlease click the link below to verify your email address:\n\n{{verification_link}}\n\nBest regards,\nThe {{app_name}} Team".to_string(),
                content_type: EmailContentType::Text,
            }),
            "notification" => Ok(EmailTemplate {
                subject: "{{subject}}".to_string(),
                body: "{{message}}".to_string(),
                content_type: EmailContentType::Text,
            }),
            _ => Err(format!("Unknown email template: {}", template_name)),
        }
    }

    /// Test the email service configuration
    #[allow(dead_code)]
    pub async fn test_connection(&self) -> EmailResult {
        info!("Testing email service connection...");

        // Test connection by sending a test email to the from address
        let test_template = EmailTemplate {
            subject: "Email Service Test".to_string(),
            body: "This is a test email to verify the email service configuration.".to_string(),
            content_type: EmailContentType::Text,
        };

        self.send_email(
            &self.config.from_email,
            Some("Test Recipient"),
            &test_template,
            None,
        )
        .await
    }
}

impl EmailConfig {
    /// Load email configuration from environment variables
    #[allow(dead_code)]
    pub fn from_env() -> Result<Self, String> {
        let service_type = env::var("EMAIL_SERVICE_TYPE").unwrap_or_else(|_| "smtp".to_string());

        let smtp_host = env::var("EMAIL_SMTP_HOST")
            .map_err(|_| "EMAIL_SMTP_HOST environment variable is required".to_string())?;

        let smtp_port = env::var("EMAIL_SMTP_PORT")
            .unwrap_or_else(|_| "587".to_string())
            .parse::<u16>()
            .map_err(|_| "EMAIL_SMTP_PORT must be a valid port number".to_string())?;

        let smtp_username = env::var("EMAIL_SMTP_USERNAME")
            .map_err(|_| "EMAIL_SMTP_USERNAME environment variable is required".to_string())?;

        let smtp_password = env::var("EMAIL_SMTP_PASSWORD")
            .map_err(|_| "EMAIL_SMTP_PASSWORD environment variable is required".to_string())?;

        let from_email = env::var("EMAIL_FROM_ADDRESS")
            .map_err(|_| "EMAIL_FROM_ADDRESS environment variable is required".to_string())?;

        let from_name =
            env::var("EMAIL_FROM_NAME").unwrap_or_else(|_| "Rext Application".to_string());

        let reply_to_email = env::var("EMAIL_REPLY_TO_ADDRESS").ok();
        let reply_to_name = env::var("EMAIL_REPLY_TO_NAME").ok();

        // Log configuration (without sensitive data)
        info!("Email service configured:");
        info!("  Service Type: {}", service_type);
        info!("  SMTP Host: {}", smtp_host);
        info!("  SMTP Port: {}", smtp_port);
        info!("  From Email: {}", from_email);
        info!("  From Name: {}", from_name);
        if reply_to_email.is_some() {
            info!("  Reply-To configured: Yes");
        }

        Ok(Self {
            service_type: EmailServiceType::from_str(&service_type).map_err(|e| e.to_string())?,
            smtp_host,
            smtp_port,
            smtp_username,
            smtp_password,
            from_email,
            from_name,
            reply_to_email,
            reply_to_name,
        })
    }
}

/// Convenience functions for common email operations
impl EmailService {
    /// Send a welcome email to a new user
    #[allow(dead_code)]
    pub async fn send_welcome_email(
        &self,
        user_email: &str,
        user_name: &str,
        app_name: &str,
    ) -> EmailResult {
        let mut variables = HashMap::new();
        variables.insert("user_name".to_string(), user_name.to_string());
        variables.insert("app_name".to_string(), app_name.to_string());

        self.send_template_email(user_email, Some(user_name), "welcome", Some(variables))
            .await
    }

    /// Send a password reset email
    #[allow(dead_code)]
    pub async fn send_password_reset_email(
        &self,
        user_email: &str,
        user_name: &str,
        reset_link: &str,
        app_name: &str,
    ) -> EmailResult {
        let mut variables = HashMap::new();
        variables.insert("user_name".to_string(), user_name.to_string());
        variables.insert("reset_link".to_string(), reset_link.to_string());
        variables.insert("app_name".to_string(), app_name.to_string());

        self.send_template_email(
            user_email,
            Some(user_name),
            "password_reset",
            Some(variables),
        )
        .await
    }

    /// Send an email verification email
    #[allow(dead_code)]
    pub async fn send_verification_email(
        &self,
        user_email: &str,
        user_name: &str,
        verification_link: &str,
        app_name: &str,
    ) -> EmailResult {
        let mut variables = HashMap::new();
        variables.insert("user_name".to_string(), user_name.to_string());
        variables.insert(
            "verification_link".to_string(),
            verification_link.to_string(),
        );
        variables.insert("app_name".to_string(), app_name.to_string());

        self.send_template_email(user_email, Some(user_name), "verification", Some(variables))
            .await
    }

    /// Send a notification email
    #[allow(dead_code)]
    pub async fn send_notification_email(
        &self,
        user_email: &str,
        user_name: Option<&str>,
        subject: &str,
        message: &str,
    ) -> EmailResult {
        let mut variables = HashMap::new();
        variables.insert("subject".to_string(), subject.to_string());
        variables.insert("message".to_string(), message.to_string());

        self.send_template_email(user_email, user_name, "notification", Some(variables))
            .await
    }
}
