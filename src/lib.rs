/*
 * TODO: Remove unnecessary cloning. For async, Send & Sync are probably needed,
 *       but not Clone...
 */

pub mod blocking {
    pub trait Resolve {
        fn resolve(&self) -> Result<crate::Resolved, std::io::Error>;
    }

    impl<T: crate::Resolver> Resolve for crate::Resolvable<T> {
        fn resolve(&self) -> Result<crate::Resolved, std::io::Error> {
            /*
             * TODO: Do actual resolving.
             */
            let dummy: &str = match self.inner {
                serde_json::Value::Object(ref map) => match map.get("foo").unwrap() {
                    serde_json::Value::String(n) => n,
                    _ => todo!(),
                },
                _ => todo!(),
            };
            let _dummy: String = self.resolver.read_file_system_blocking(&dummy)?;
            Ok(crate::Resolved(self.inner.clone()))
        }
    }
}

#[cfg(feature = "tokio")]
pub mod tokio {
    pub trait Resolve {
        fn resolve(
            &self,
        ) -> impl std::future::Future<Output = Result<crate::Resolved, std::io::Error>>;
    }

    impl<T: crate::Resolver + Clone> Resolve for crate::Resolvable<T> {
        fn resolve(
            &self,
        ) -> impl std::future::Future<Output = Result<crate::Resolved, std::io::Error>> {
            let inner = self.inner.clone();
            let resolver = self.resolver.clone();
            async move {
                /*
                 * TODO: Do actual resolving.
                 */
                let dummy: &str = match self.inner {
                    serde_json::Value::Object(ref map) => match map.get("foo").unwrap() {
                        serde_json::Value::String(n) => n,
                        _ => todo!(),
                    },
                    _ => todo!(),
                };
                let _dummy: String = self.resolver.read_file_system_tokio(&dummy).await?;
                Ok(crate::Resolved(inner))
            }
        }
    }
}

pub struct Resolvable<T: Resolver = DefaultResolver> {
    inner: serde_json::Value,
    resolver: T,
}

impl Resolvable {
    pub fn parse_json5(buffer: &str) -> Result<Self, serde_json5::Error> {
        let inner: serde_json::Value = serde_json5::from_str(buffer)?;
        Ok(Self {
            inner,
            resolver: DefaultResolver,
        })
    }

    #[cfg(feature = "toml")]
    pub fn parse_toml(buffer: &str) -> Result<Self, toml::de::Error> {
        let inner: serde_json::Value = toml::from_str(buffer)?;
        Ok(Self {
            inner,
            resolver: DefaultResolver,
        })
    }
}

impl<T: Resolver> Resolvable<T> {
    pub fn set_resolver<R: Resolver>(self, resolver: R) -> Resolvable<R> {
        Resolvable {
            inner: self.inner,
            resolver,
        }
    }

    pub fn deserialize<D: serde::de::DeserializeOwned>(&self) -> Result<D, serde_json::Error> {
        serde_json::from_value(self.inner.clone())
    }
}

pub struct Resolved(serde_json::Value);

impl Resolved {
    pub fn deserialize<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_value(self.0.clone())
    }
}

pub trait Resolver {
    fn read_file_system_blocking(&self, path: &str) -> Result<String, std::io::Error>;

    #[cfg(feature = "tokio")]
    fn read_file_system_tokio(
        &self,
        path: &str,
    ) -> impl std::future::Future<Output = Result<String, std::io::Error>>;
}

#[derive(Clone)]
pub struct DefaultResolver;

impl Resolver for DefaultResolver {
    fn read_file_system_blocking(&self, path: &str) -> Result<String, std::io::Error> {
        std::fs::read_to_string(path)
    }

    #[cfg(feature = "tokio")]
    fn read_file_system_tokio(
        &self,
        path: &str,
    ) -> impl std::future::Future<Output = Result<String, std::io::Error>> {
        async move { ::tokio::fs::read_to_string(path).await }
    }
}
