use super::*;

pub struct Resolver {}

impl Resolver {
    #[expect(clippy::unused_async)]
    pub async fn txt_lookup<S: AsRef<str>>(_host: S) -> Result<Vec<String>, ResolverError> {
        Err(ResolverError::Generic(
            "wasm does not support txt lookup".to_owned(),
        ))
    }

    #[expect(clippy::unused_async)]
    pub async fn ptr_lookup(_ip_addr: IpAddr) -> Result<String, ResolverError> {
        Err(ResolverError::Generic(
            "wasm does not support ptr lookup".to_owned(),
        ))
    }
}
