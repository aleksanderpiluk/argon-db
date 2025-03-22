use super::segment::Segment;

#[derive(Debug)]
pub struct WALState {
    pub allocating: Segment,
}
