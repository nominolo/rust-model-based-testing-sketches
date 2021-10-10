use proptest::{collection, prelude::*};

use roaring::RoaringBitmap;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitmapModel {
    data: BTreeSet<u32>,
}

impl BitmapModel {
    pub fn new() -> Self {
        BitmapModel {
            data: BTreeSet::default(),
        }
    }

    pub fn insert(&mut self, value: u32) -> bool {
        self.data.insert(value)
    }

    pub fn remove(&mut self, value: u32) -> bool {
        self.data.remove(&value)
    }

    pub fn contains(&self, value: u32) -> bool {
        self.data.contains(&value)
    }
}

#[derive(Debug)]
pub enum Action {
    Insert(u32),
    Remove(u32),
    Contains(u32),
}


pub fn apply_and_check_result(
    action: &Action,
    model: &mut BitmapModel,
    actual: &mut RoaringBitmap,
) -> Result<(), TestCaseError> {
    match action {
        Action::Remove(item) => {
            check_result(action, model.remove(*item), actual.remove(*item))
        },
        Action::Insert(item) => {
            check_result(action, model.insert(*item), actual.insert(*item))
        },
        Action::Contains(item) => {
            check_result(action, model.contains(*item), actual.contains(*item))
        },
    }
}

fn check_result<T: std::fmt::Debug + PartialEq>(
    action: &Action,
    model_result: T,
    actual_result: T,
) -> Result<(), TestCaseError> {
    if actual_result != model_result {
        Err(TestCaseError::Fail(
            format!(
                "Action: {:?} Model result: {:?} Actual result: {:?}",
                action, model_result, actual_result
            )
            .into(),
        ))
    } else {
        Ok(())
    }
}

fn arb_insert(max_val: u32) -> impl Strategy<Value = Action> {
    (0..max_val).prop_map(Action::Insert)
}

fn arb_remove(max_val: u32) -> impl Strategy<Value = Action> {
    (0..max_val).prop_map(Action::Remove)
}

fn arb_contains(max_val: u32) -> impl Strategy<Value = Action> {
    (0..max_val).prop_map(Action::Contains)
}

fn arb_simple_action(max_val: u32)-> impl Strategy<Value = Action> {
    prop_oneof![
        3 => arb_insert(max_val),
        3 => arb_remove(max_val),
        1 => arb_contains(max_val)
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]
    #[test]
    fn single_items(actions in collection::vec(arb_simple_action(50), 1..50)) {
        let mut model = BitmapModel::new();
        let mut actual = RoaringBitmap::new();
        //println!("{:?}", actions);
        for action in actions {
            //let action = Action::Push(a as i32, a as i64);
            if true /*pre_condition(&model, &action)*/ {
                apply_and_check_result(&action, &mut model, &mut actual)?;
            }
        }
    }
}