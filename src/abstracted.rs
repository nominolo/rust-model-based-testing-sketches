use proptest::prelude::*;

pub trait ModelProps<M, A, S> {
    fn initial_model() -> M;
    fn check_precondition(action: &A, model: &M) -> bool;
    fn apply_action_and_check_result(
        action: A,
        model: &mut M,
        system: &mut S,
    ) -> Result<(), TestCaseError>;
}
