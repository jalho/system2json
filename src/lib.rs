pub mod blocking {
    pub trait Resolve {
        fn resolve(&self) -> Result<crate::Resolved, std::io::Error>;
    }

    impl Resolve for crate::Resolvable {
        fn resolve(&self) -> Result<crate::Resolved, std::io::Error> {
            /*
             * TODO: Implement resolving!
             */
            Ok(crate::Resolved(self.0.clone()))
        }
    }
}

pub mod tokio {
    pub trait Resolve {
        fn resolve(
            &self,
        ) -> impl std::future::Future<Output = Result<crate::Resolved, tokio::io::Error>>;
    }

    impl Resolve for crate::Resolvable {
        fn resolve(
            &self,
        ) -> impl std::future::Future<Output = Result<crate::Resolved, tokio::io::Error>> {
            /*
             * TODO: Implement resolving!
             */
            async { Ok(crate::Resolved(self.0.clone())) }
        }
    }
}

pub struct Resolvable(serde_json::Value);

impl Resolvable {
    pub fn parse_json5(buffer: &str) -> Result<Self, serde_json5::Error> {
        let intermediate: serde_json::Value = serde_json5::from_str(buffer)?;
        Ok(Self(intermediate))
    }

    pub fn deserialize<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_value(self.0.clone())
    }
}

pub struct Resolved(serde_json::Value);

impl Resolved {
    pub fn deserialize<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_value(self.0.clone())
    }
}
