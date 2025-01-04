use std::{convert::TryFrom, fmt::Display, net::TcpStream};
use thiserror::Error;
use windows::core::GUID;

use super::super::api::Result as ApiResult;
use crate::{CoreDistributionInformation, WSLContext, WSLSessionInformation};

#[derive(Debug, Clone, Copy)]
pub enum DistributionID {
    System,
    User(GUID),
}

#[derive(Debug, Error)]
#[error("Cannot convert System distribution to GUID.")]
pub struct ConversionError;

impl TryFrom<DistributionID> for GUID {
    type Error = ConversionError;

    fn try_from(value: DistributionID) -> Result<Self, Self::Error> {
        match value {
            DistributionID::User(id) => Ok(id),
            DistributionID::System => Err(ConversionError),
        }
    }
}

impl From<GUID> for DistributionID {
    fn from(value: GUID) -> Self {
        Self::User(value)
    }
}

impl<T: CoreDistributionInformation> From<T> for DistributionID {
    fn from(value: T) -> Self {
        (*value.id()).into()
    }
}

impl From<Option<GUID>> for DistributionID {
    fn from(value: Option<GUID>) -> Self {
        match value {
            Some(id) => Self::User(id),
            None => Self::System,
        }
    }
}

impl From<DistributionID> for Option<GUID> {
    fn from(value: DistributionID) -> Self {
        match value {
            DistributionID::System => None,
            DistributionID::User(id) => Some(id),
        }
    }
}

impl Display for DistributionID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DistributionID::System => f.write_str("System"),
            DistributionID::User(id) => std::fmt::Debug::fmt(id, f),
        }
    }
}

#[derive(Clone)]
pub struct Command<'a> {
    context: &'static WSLContext,
    args: Vec<&'a str>,
    path: &'a str,
    distribution_id: DistributionID,
    session: &'a WSLSessionInformation<'a>,
}

impl<'a> Command<'a> {
    pub fn new(
        context: &'static WSLContext,
        session: &'a WSLSessionInformation<'a>,
        program: &'a str,
    ) -> Self {
        Self {
            context,
            args: vec![program],
            path: program,
            distribution_id: DistributionID::System,
            session,
        }
    }

    pub fn get_path(&self) -> &'a str {
        self.path
    }

    pub fn arg0(&mut self, arg0: &'a str) -> &mut Self {
        self.args[0] = arg0;
        self
    }

    pub fn get_arg0(&self) -> &str {
        self.args[0]
    }

    pub fn is_standard_arg_0(&self) -> bool {
        self.path == self.get_arg0()
    }

    pub fn reset_arg0(&mut self) -> &mut Self {
        self.args[0] = self.path;
        self
    }

    pub fn arg(&mut self, arg: &'a str) -> &mut Self {
        self.args.push(arg);
        self
    }

    pub fn args<I: IntoIterator<Item = &'a str>>(&mut self, args: I) -> &mut Self {
        self.args.extend(args);
        self
    }

    pub fn get_args(&self) -> impl ExactSizeIterator<Item = &str> {
        self.args[1..].iter().copied()
    }

    pub fn distribution_id(&mut self, distribution_id: DistributionID) -> &mut Self {
        self.distribution_id = distribution_id;
        self
    }

    pub fn reset_distribution_id(&mut self) -> &mut Self {
        self.distribution_id = DistributionID::System;
        self
    }

    pub fn get_distribution_id(&self) -> DistributionID {
        self.distribution_id
    }

    pub fn execute(&mut self) -> ApiResult<TcpStream> {
        let stream = match self.distribution_id {
            DistributionID::System => {
                self.context
                    .api
                    .execute_binary(self.session, self.path, self.args.as_slice())?
            }
            DistributionID::User(id) => self.context.api.execute_binary_in_distribution(
                self.session,
                &id,
                self.path,
                self.args.as_slice(),
            )?,
        };
        Ok(stream)
    }
}
