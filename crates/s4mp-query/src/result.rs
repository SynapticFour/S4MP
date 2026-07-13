use s4mp_model::Node;

#[derive(Clone, Debug, Default)]
pub struct QueryResult {
    pub nodes: Vec<Node>,
}
