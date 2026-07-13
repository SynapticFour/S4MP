use s4mp_model::NodeKind;

/// Query AST root. Full S4QL parser to be added in Phase 2.
#[derive(Clone, Debug)]
pub struct Query {
    pub expr: QueryExpr,
}

#[derive(Clone, Debug)]
pub enum QueryExpr {
    MatchNodes { kind: Option<NodeKind> },
    All,
}
