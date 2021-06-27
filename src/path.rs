use crate::domain::Domain;


pub struct Path {
    local_part: Option<String>,
    domain: Option<Domain>,
}