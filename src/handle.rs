use std::sync::Arc;


pub struct EditorHandle(pub(crate) Arc<EditorHandleInner>);


pub(crate) struct EditorHandleInner {}
impl EditorHandleInner {
    pub(crate) fn new() -> Arc<Self> { Arc::new(Self {
    }) }
}
