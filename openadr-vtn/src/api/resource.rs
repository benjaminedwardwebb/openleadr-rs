use std::sync::Arc;

use axum::{
    extract::{Path, State},
    Json,
};
use openadr_wire::problem::Problem;
use openadr_wire::ven::VenId;
use reqwest::StatusCode;
use serde::Deserialize;
use tracing::{info, trace};
use validator::{Validate, ValidationError};

use openadr_wire::{
    resource::{Resource, ResourceContent, ResourceId},
    target::TargetLabel,
};

use crate::{
    api::{AppResponse, ValidatedJson, ValidatedQuery},
    data_source::ResourceCrud,
    error::AppError,
    jwt::User,
};

fn has_write_permission(User(claims): &User, ven_id: &VenId) -> Result<(), AppError> {
    if claims.is_ven_manager() {
        return Ok(());
    }

    if claims.is_ven() && claims.ven_ids().contains(ven_id) {
        return Ok(());
    }

    Err(AppError::Forbidden(
        "User not authorized to access this resource",
    ))
}

/// search ven resources
///
/// Return the ven resources specified by venID specified in path.
#[utoipa::path(
    get,
    path = "/vens/{venID}/resources",
    responses(
        (status = 200, description = "OK.", body = Vec<Resource>),
        (status = 400, description = "Bad Request.", body = Problem),
        (status = 403, description = "Forbidden.", body = Problem),
        (status = 404, description = "Not Found.", body = Problem),
        (status = 500, description = "Internal Server Error.", body = Problem),
    ),
    params(
        ("venID" = VenId, Path, description = "Numeric ID of ven."),
        ("targetType" = Option<String>, Query, description = "Indicates targeting type, e.g. GROUP"),
        ("targetValues" = Option<Vec<String>>, Query, description = "List of target values, e.g. group names"),
        ("skip" = Option<i64>, Query, description = "number of records to skip for pagination.", style = Form, explode, minimum = 0),
        ("limit" = Option<i64>, Query, description = "maximum number of records to return.", style = Form, explode, minimum = 1, maximum = 50)
    )
)]
pub async fn get_all(
    State(resource_source): State<Arc<dyn ResourceCrud>>,
    Path(ven_id): Path<VenId>,
    ValidatedQuery(query_params): ValidatedQuery<QueryParams>,
    user: User,
) -> AppResponse<Vec<Resource>> {
    has_write_permission(&user, &ven_id)?;
    trace!(?query_params);

    let resources = resource_source
        .retrieve_all(ven_id, &query_params, &user)
        .await?;

    Ok(Json(resources))
}

#[utoipa::path(
    get,
    path = "/vens/{venID}/resources/{resourceID}",
    params(
        ("venID" = str, Path, description = "object ID of the associated ven."),
        ("resourceID" = str, Path, description = "object ID of the resource.")
    ),
    responses(
        (status = 200, description = "Return the ven resource specified by venID and resourceID specified in path.", body = Resource),
        (status = 400, description = "TODO", body = Resource)
    ),
)]
pub async fn get(
    State(resource_source): State<Arc<dyn ResourceCrud>>,
    Path((ven_id, id)): Path<(VenId, ResourceId)>,
    user: User,
) -> AppResponse<Resource> {
    has_write_permission(&user, &ven_id)?;
    let ven = resource_source.retrieve(&id, ven_id, &user).await?;

    Ok(Json(ven))
}

pub async fn add(
    State(resource_source): State<Arc<dyn ResourceCrud>>,
    user: User,
    Path(ven_id): Path<VenId>,
    ValidatedJson(new_resource): ValidatedJson<ResourceContent>,
) -> Result<(StatusCode, Json<Resource>), AppError> {
    has_write_permission(&user, &ven_id)?;
    let ven = resource_source.create(new_resource, ven_id, &user).await?;

    Ok((StatusCode::CREATED, Json(ven)))
}

#[utoipa::path(
    put,
    path = "/vens/{venID}/resources/{resourceID}",
    responses(
        (status = 200, description = "Update the ven resource specified by venID and resourceID specified in path.", body = Resource)
    ),
    params(
        ("venID" = str, Path, description = "object ID of the associated ven."),
        ("resourceID" = str, Path, description = "object ID of the resource.")
    )
)]
pub async fn edit(
    State(resource_source): State<Arc<dyn ResourceCrud>>,
    Path((ven_id, id)): Path<(VenId, ResourceId)>,
    user: User,
    ValidatedJson(content): ValidatedJson<ResourceContent>,
) -> AppResponse<Resource> {
    has_write_permission(&user, &ven_id)?;
    let resource = resource_source.update(&id, ven_id, content, &user).await?;

    info!(%resource.id, resource.resource_name=resource.content.resource_name, "resource updated");

    Ok(Json(resource))
}

#[utoipa::path(
    delete,
    path = "/vens/{venID}/resources/{resourceID}",
    responses(
        (status = 200, description = "Delete the ven resource specified by venID and resourceID specified in path.", body = Resource)
    ),
    params(
        ("venID" = str, Path, description = "object ID of the associated ven."),
        ("resourceID" = str, Path, description = "object ID of the resource.")
    )
)]
pub async fn delete(
    State(resource_source): State<Arc<dyn ResourceCrud>>,
    Path((ven_id, id)): Path<(VenId, ResourceId)>,
    user: User,
) -> AppResponse<Resource> {
    has_write_permission(&user, &ven_id)?;
    let resource = resource_source.delete(&id, ven_id, &user).await?;
    info!(%id, "deleted resource");
    Ok(Json(resource))
}

#[derive(Deserialize, Validate, Debug)]
#[validate(schema(function = "validate_target_type_value_pair"))]
#[serde(rename_all = "camelCase")]
pub struct QueryParams {
    pub(crate) target_type: Option<TargetLabel>,
    pub(crate) target_values: Option<Vec<String>>,
    #[serde(default)]
    #[validate(range(min = 0))]
    pub(crate) skip: i64,
    #[validate(range(min = 1, max = 50))]
    #[serde(default = "get_50")]
    pub(crate) limit: i64,
}

fn validate_target_type_value_pair(query: &QueryParams) -> Result<(), ValidationError> {
    if query.target_type.is_some() == query.target_values.is_some() {
        Ok(())
    } else {
        Err(ValidationError::new("targetType and targetValues query parameter must either both be set or not set at the same time."))
    }
}

fn get_50() -> i64 {
    50
}

#[cfg(test)]
mod test {
    use crate::{api::test::ApiTest, jwt::AuthRole};
    use axum::body::Body;
    use openadr_wire::{
        problem::Problem,
        resource::{Resource, ResourceContent},
    };
    use reqwest::{Method, StatusCode};
    use sqlx::PgPool;

    fn default() -> ResourceContent {
        ResourceContent {
            object_type: Default::default(),
            resource_name: "".to_string(),
            attributes: None,
            targets: None,
        }
    }

    #[sqlx::test(fixtures("users", "vens", "resources"))]
    async fn test_get_all(db: PgPool) {
        let test = ApiTest::new(db.clone(), vec![AuthRole::VenManager]);

        let (status, resources) = test
            .request::<Vec<Resource>>(Method::GET, "/vens/ven-1/resources", Body::empty())
            .await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(resources.len(), 2);

        let (status, resources) = test
            .request::<Vec<Resource>>(Method::GET, "/vens/ven-2/resources", Body::empty())
            .await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(resources.len(), 3);

        // test with ven user
        let test = ApiTest::new(db, vec![AuthRole::VEN("ven-1".parse().unwrap())]);

        let (status, resources) = test
            .request::<Vec<Resource>>(Method::GET, "/vens/ven-1/resources", Body::empty())
            .await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(resources.len(), 2);

        let (status, _) = test
            .request::<serde_json::Value>(Method::GET, "/vens/ven-2/resources", Body::empty())
            .await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[sqlx::test(fixtures("users", "vens", "resources"))]
    async fn get_all_filtered(db: PgPool) {
        let test = ApiTest::new(db.clone(), vec![AuthRole::VenManager]);

        let (status, resources) = test
            .request::<Vec<Resource>>(Method::GET, "/vens/ven-1/resources?skip=1", Body::empty())
            .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(resources.len(), 1);

        let (status, resources) = test
            .request::<Vec<Resource>>(Method::GET, "/vens/ven-1/resources?limit=1", Body::empty())
            .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(resources.len(), 1);

        let (status, resources) = test
            .request::<Vec<Resource>>(
                Method::GET,
                "/vens/ven-1/resources?targetType=RESOURCE_NAME&targetValues=resource-1-name",
                Body::empty(),
            )
            .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(resources.len(), 1);

        let (status, resources) = test
            .request::<Vec<Resource>>(
                Method::GET,
                "/vens/ven-1/resources?targetType=RESOURCE_NAME&targetValues=resource-2-name",
                Body::empty(),
            )
            .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(resources.len(), 0);
    }

    #[sqlx::test(fixtures("users", "vens", "resources"))]
    async fn get_single_resource(db: PgPool) {
        let test = ApiTest::new(db.clone(), vec![AuthRole::VenManager]);

        let (status, resource) = test
            .request::<Resource>(
                Method::GET,
                "/vens/ven-1/resources/resource-1",
                Body::empty(),
            )
            .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(resource.id.as_str(), "resource-1");

        // test with ven user
        let test = ApiTest::new(db, vec![AuthRole::VEN("ven-1".parse().unwrap())]);

        let (status, resource) = test
            .request::<Resource>(
                Method::GET,
                "/vens/ven-1/resources/resource-1",
                Body::empty(),
            )
            .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(resource.id.as_str(), "resource-1");

        let (status, _) = test
            .request::<Problem>(
                Method::GET,
                "/vens/ven-1/resources/resource-2",
                Body::empty(),
            )
            .await;
        assert_eq!(status, StatusCode::NOT_FOUND);

        let (status, _) = test
            .request::<Problem>(
                Method::GET,
                "/vens/ven-2/resources/resource-2",
                Body::empty(),
            )
            .await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[sqlx::test(fixtures("users", "vens", "resources"))]
    async fn add_edit_delete(db: PgPool) {
        let test = ApiTest::new(db.clone(), vec![AuthRole::VenManager]);

        let (status, resource) = test
            .request::<Resource>(
                Method::POST,
                "/vens/ven-1/resources",
                Body::from(r#"{"resourceName":"new-resource"}"#),
            )
            .await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(resource.content.resource_name, "new-resource");

        let resource_id = resource.id.as_str();

        let (status, resource) = test
            .request::<Resource>(
                Method::PUT,
                &format!("/vens/ven-1/resources/{resource_id}"),
                Body::from(r#"{"resourceName":"updated-resource"}"#),
            )
            .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(resource.content.resource_name, "updated-resource");

        let (status, resource) = test
            .request::<Resource>(
                Method::GET,
                &format!("/vens/ven-1/resources/{resource_id}"),
                Body::empty(),
            )
            .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(resource.content.resource_name, "updated-resource");

        let (status, _) = test
            .request::<Resource>(
                Method::DELETE,
                &format!("/vens/ven-1/resources/{resource_id}"),
                Body::empty(),
            )
            .await;
        assert_eq!(status, StatusCode::OK);

        let (status, _) = test
            .request::<Problem>(
                Method::GET,
                &format!("/vens/ven-1/resources/{resource_id}"),
                Body::empty(),
            )
            .await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[sqlx::test(fixtures("users", "vens"))]
    async fn name_constraint_validation(db: PgPool) {
        let test = ApiTest::new(db, vec![AuthRole::AnyBusiness]);

        let resources = [
            ResourceContent {
                resource_name: "".to_string(),
                ..default()
            },
            ResourceContent {
                resource_name: "This is more than 128 characters long and should be rejected This is more than 128 characters long and should be rejected asdfasd".to_string(),
                ..default()
            },
        ];

        for resource in &resources {
            let (status, error) = test
                .request::<Problem>(
                    Method::POST,
                    "/vens/ven-1/resources",
                    Body::from(serde_json::to_vec(&resource).unwrap()),
                )
                .await;

            assert_eq!(status, StatusCode::BAD_REQUEST);
            assert!(error
                .detail
                .unwrap()
                .contains("outside of allowed range 1..=128"))
        }
    }
}
