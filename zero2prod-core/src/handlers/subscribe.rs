use tracing::{info, instrument, Span};

use crate::domain::{Document, NewSubscriber};
use crate::error::CoreResult;
use crate::repository::SubscriptionRepository;
use crate::service::email_service::EmailService;

#[instrument(name = "Subscription", skip_all)]
pub async fn subscribe<S, E>(
    subscriber_repo: &S,
    email_client: &E,
    new_subscriber: NewSubscriber,
    confirmation_email: Document,
) -> CoreResult<()>
where
    S: SubscriptionRepository,
    E: EmailService,
{
    Span::current()
        .record("subscriber_email", new_subscriber.email.as_ref())
        .record("subscriber_name", new_subscriber.name.as_ref());

    info!("Adding a new subscriber");
    subscriber_repo.create(&new_subscriber).await?;

    info!("Sending confirmation email");
    email_client
        .send_email(new_subscriber.email.as_ref(), confirmation_email)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {

    use fake::faker::{internet::en::SafeEmail, lorem::en::Sentence, name::en::Name};
    use fake::Fake;
    use mockall::predicate::eq;

    use crate::{
        domain::DocumentKind, error::CoreError, repository::MockSubscriptionRepository,
        service::email_service::MockEmailService,
    };

    use super::*;

    fn random_subscriber() -> NewSubscriber {
        NewSubscriber {
            email: SafeEmail().fake::<String>().parse().unwrap(),
            name: Name().fake::<String>().parse().unwrap(),
        }
    }

    fn random_confirmation_email() -> Document {
        Document {
            title: Sentence(1..2).fake::<String>(),
            kind: DocumentKind::Confirmation {
                confirmation_link: "https://my.link.com".to_owned(),
            },
        }
    }

    #[test]
    fn subscribe_when_there_is_a_database_error() {
        let new_subscriber = random_subscriber();
        let email = random_confirmation_email();

        let mut mock_repo = MockSubscriptionRepository::new();
        mock_repo
            .expect_create()
            .returning(|_| Err(CoreError::EmailAlreadyExists));

        let mut mock_email_service = MockEmailService::new();
        mock_email_service.expect_send_email().times(0);

        tokio_test::block_on(async {
            assert_eq!(
                subscribe(&mock_repo, &mock_email_service, new_subscriber, email).await,
                Err(CoreError::EmailAlreadyExists)
            );
        })
    }

    #[test]
    fn subscribe_when_there_is_a_mail_service_error() {
        let new_subscriber = random_subscriber();
        let email = random_confirmation_email();

        let mut mock_repo = MockSubscriptionRepository::new();
        mock_repo.expect_create().times(1).returning(|_| Ok(()));

        let mut mock_email_service = MockEmailService::new();
        mock_email_service
            .expect_send_email()
            .times(1)
            .returning(|_, _| Err(CoreError::Unexpected("failed to send mail".into())));

        tokio_test::block_on(async {
            assert_eq!(
                subscribe(&mock_repo, &mock_email_service, new_subscriber, email).await,
                Err(CoreError::Unexpected("failed to send mail".into()))
            );
        })
    }

    #[test]
    fn subscribe_nominal_case() {
        let new_subscriber = random_subscriber();
        let email = random_confirmation_email();
        let expected_recipient: String = new_subscriber.email.as_str().to_owned();

        let mut mock_repo = MockSubscriptionRepository::new();
        mock_repo
            .expect_create()
            .times(1)
            .with(eq(new_subscriber.clone()))
            .returning(|_| Ok(()));

        let mut mock_email_service = MockEmailService::new();
        mock_email_service
            .expect_send_email()
            .times(1)
            .with(eq(expected_recipient), eq(email.clone()))
            .returning(|_, _| Err(CoreError::Unexpected("failed to send mail".into())));

        tokio_test::block_on(async {
            assert_eq!(
                subscribe(&mock_repo, &mock_email_service, new_subscriber, email).await,
                Err(CoreError::Unexpected("failed to send mail".into()))
            );
        })
    }
}
