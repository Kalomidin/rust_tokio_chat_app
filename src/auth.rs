use crate::db::user::get_user_by_id;
use crate::errors::{db_error_to_service_error, internal_error_to_service_error};
use crate::{errors::ServiceError, ConnectionPool};
use axum::{
  extract::State,
  headers::{authorization::Bearer, Authorization},
  http::{Request, StatusCode},
  middleware::Next,
  response::Response,
  TypedHeader,
};
use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
struct Claim {
  user_id: i64, // user id
  exp: usize,   // expiry time
  iat: usize,   // issued at
}

// TODO: Move them to .env file
const BEARER: &str = "Bearer ";
const JWT_SECRET: &[u8] = b"secret";
const JWT_TOKEN_DURATION_IN_HOURS: i64 = 24;
const ALGORIITHM: Algorithm = Algorithm::HS256;

pub fn create_jwt(user_id: i64) -> Result<String, ServiceError> {
  let expiration = Utc::now()
    .checked_add_signed(chrono::Duration::hours(JWT_TOKEN_DURATION_IN_HOURS))
    .expect("valid timestamp")
    .timestamp();

  let claims = Claim {
    user_id: user_id,
    exp: expiration as usize,
    iat: Utc::now().timestamp() as usize,
  };
  let header = Header::new(ALGORIITHM);
  encode(&header, &claims, &EncodingKey::from_secret(JWT_SECRET)).map_err(|_| {
    ServiceError::new(
      StatusCode::INTERNAL_SERVER_ERROR,
      "Failed to create jwt token",
    )
  })
}

fn validate_token(token: &str) -> Result<Claim, jsonwebtoken::errors::Error> {
  let token = token.trim_start_matches(BEARER);
  let decoded = decode::<Claim>(
    token,
    &DecodingKey::from_secret(JWT_SECRET),
    &Validation::new(ALGORIITHM),
  );
  match decoded {
    Ok(claim) => {
      let now = Utc::now().timestamp() as usize;
      if now > claim.claims.exp {
        return Err((jsonwebtoken::errors::ErrorKind::ExpiredSignature).into());
      }
      Ok(claim.claims)
    }
    Err(e) => Err(e),
  }
}

pub async fn guard<T>(
  State(pool): State<ConnectionPool>,
  TypedHeader(token): TypedHeader<Authorization<Bearer>>,
  mut request: Request<T>,
  next: Next<T>,
) -> Result<Response, ServiceError> {
  let token = token.token().to_owned();
  let _claim = validate_token(&token);
  match _claim {
    Ok(claim) => {
      let user_id = claim.user_id;
      let mut _conn = pool.get().await.map_err(internal_error_to_service_error)?;
      get_user_by_id(&mut _conn, user_id)
        .await
        .map_err(db_error_to_service_error)?;

      request.extensions_mut().insert(claim.user_id);
    }
    Err(_) => {
      return Err(ServiceError::new(StatusCode::UNAUTHORIZED, "Unauthorized"));
    }
  }

  Ok(next.run(request).await)
}
