pub(super) mod models;
pub(super) mod params;


pub trait MapParams {
    fn get_pairs(self) -> Option<std::collections::HashMap<&'static str, crate::database::repository::TypeTable>>;
}