use s4mp_model::NodeId;

#[derive(Clone, Debug)]
pub struct Finding {
    pub message: String,
    pub related_node: Option<NodeId>,
}
