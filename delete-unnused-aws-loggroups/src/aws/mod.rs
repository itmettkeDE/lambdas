mod codebuild;
mod lambda;
mod logs;

pub use codebuild::CodeBuild;
pub use lambda::Lambda;
pub use logs::Logs;

/// Checks whether the given result is a throttling error
/// and waits for 250 ms if it is
async fn is_wait_and_repeat<D: Send + Sync, E: std::fmt::Debug + Send + Sync>(
    error: &Result<D, rusoto_core::RusotoError<E>>,
) -> bool {
    if let Err(rusoto_core::RusotoError::Unknown(rusoto_core::request::BufferedHttpResponse {
        ref status,
        ref body,
        ..
    })) = *error
    {
        let cooldown = match status.as_u16() {
            400 => {
                let search = b"ThrottlingException";
                body.as_ref().windows(search.len()).any(|sub| sub == search)
            }
            429 => {
                let search = b"Too Many Requests";
                body.as_ref().windows(search.len()).any(|sub| sub == search)
            }
            _ => false,
        };
        if cooldown {
            println!("Info: Cooling down to prevent request limits");
            tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
            return true;
        }
    }
    false
}
