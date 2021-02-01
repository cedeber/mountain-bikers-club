use crate::users::api::SearchForm;
use crate::utils::redirect_to;
use actix_web::{web, HttpResponse};

// => /api/user/search
pub async fn search(form: web::Form<SearchForm>) -> HttpResponse {
    if form.query.is_empty() {
        redirect_to(&*format!("/{}?tab={}", form.member, form.tab))
    } else {
        redirect_to(&*format!(
            "/{}?tab={}&search={}",
            form.member, form.tab, form.query
        ))
    }
}
