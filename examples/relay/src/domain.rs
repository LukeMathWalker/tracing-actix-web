use actix_web::web::Query;

#[derive(Debug, Clone)]
pub enum ConversionError {
    Invalid(String),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Gender {
    Female,
    Male,
}

#[derive(Debug, serde::Serialize)]
pub struct User {
    pub name: String,
    pub gender: Gender,
}

#[derive(Debug, serde::Deserialize)]
pub struct Filter {
    pub gender: Option<Gender>,
}

impl From<Option<Query<Filter>>> for super::protocol::users::Filter {
    fn from(input: Option<Query<Filter>>) -> Self {
        Self {
            gender: input.and_then(|query| query.into_inner().gender).and_then(
                |gender| match gender {
                    Gender::Female => Some(super::protocol::users::Gender::Female as i32),
                    Gender::Male => Some(super::protocol::users::Gender::Male as i32),
                },
            ),
        }
    }
}

impl std::convert::TryFrom<super::protocol::users::Users> for Vec<User> {
    type Error = ConversionError;

    fn try_from(input: super::protocol::users::Users) -> Result<Self, Self::Error> {
        let mut users = Vec::<User>::with_capacity(input.users.len());
        for user in input.users {
            let gender = match user.gender {
                female if female == super::protocol::users::Gender::Female as i32 => Gender::Female,
                male if male == super::protocol::users::Gender::Male as i32 => Gender::Male,
                unknown => {
                    return Err(ConversionError::Invalid(format!(
                        "unknown gender ({})",
                        unknown
                    )))
                }
            };
            users.push(User {
                name: user.name,
                gender,
            })
        }
        Ok(users)
    }
}
