use std::sync::Arc;

use priority_queue::PriorityQueue;
use proptest::prelude::*;

pub mod abstracted;
pub mod checkroaring;

fn main() {
    println!("Hello, world!");
}

/*
fn parse_date(s: &str) -> Option<(u32, u32, u32)> {
    if 10 != s.len() {
        return None;
    }
    if !s.is_ascii() {
        return None;
    }
    if "-" != &s[4..5] || "-" != &s[7..8] {
        return None;
    }

    let year = &s[0..4];
    let month = &s[6..7];
    let day = &s[8..10];

    year.parse::<u32>().ok().and_then(|y| {
        month
            .parse::<u32>()
            .ok()
            .and_then(|m| day.parse::<u32>().ok().map(|d| (y, m, d)))
    })
}

#[test]
fn test_parse_date() {
    assert_eq!(None, parse_date("2017-06-1"));
    assert_eq!(None, parse_date("2017-06-170"));
    assert_eq!(None, parse_date("2017006-17"));
    assert_eq!(None, parse_date("2017-06017"));
    assert_eq!(Some((2017, 06, 17)), parse_date("2017-06-17"));
}

#[test]
fn test_unicode_gibberish() {
    assert_eq!(None, parse_date(" 00﹨𞹔"));
}

proptest! {
    #[test]
    fn doesnt_crash(s in "\\PC*") {
        parse_date(&s);
    }

    #[test]
    fn parses_all_valid_dates(s in "[0-9]{4}-[0-9]{2}-[0-9]{2}") {
        parse_date(&s).unwrap();
    }

    #[test]
    fn parses_date_back_to_original(y in 0u32..10000,
                                    m in 1u32..13, d in 1u32..32) {
        println!("y = {}, m = {}, d = {}", y, m, d);
        let (y2, m2, d2) = parse_date(
            &format!("{:04}-{:02}-{:02}", y, m, d)).unwrap();
        // prop_assert_eq! is basically the same as assert_eq!, but doesn't
        // cause a bunch of panic messages to be printed on intermediate
        // test failures. Which one to use is largely a matter of taste.
        prop_assert_eq!((y, m, d), (y2, m2, d2));
    }
}
*/
#[derive(Debug)]
pub struct PqueueModel {
    items: Vec<(i32, String)>,
}

impl PqueueModel {
    pub fn new() -> Self {
        PqueueModel { items: Vec::new() }
    }

    // pub fn push(&mut self, item: i64, prio: i32) -> Option<i32> {
    //     self.items.push((prio, item));
    //     None
    // }
    fn find_item_index(&self, item: &str) -> Option<usize> {
        self.items
            .iter()
            .enumerate()
            .find(|(_idx, (_prio, it))| *it == item)
            .map(|(idx, _)| idx)
    }

    pub fn push(&mut self, item: String, prio: i32) -> Option<i32> {
        if let Some(idx) = self.find_item_index(&item)  {
            let old_prio = self.items[idx].0;
            self.items[idx] = (prio, item);
            Some(old_prio)
        } else {
            self.items.push((prio, item));
            None
        }
    }

    pub fn remove(&mut self, item: &str) -> Option<(String, i32)> {
        if let Some(idx) = self.find_item_index(item) {
            let (prio, item) = self.items.remove(idx);
            Some((item, prio))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Push(i32, String),
    Remove(String),
}

pub fn pre_condition(model: &PqueueModel, action: &Action) -> bool {
    match action {
        Action::Push(_, _) => true,
        Action::Remove(_) => true,
    }
}

pub fn apply_and_check_result(
    action: &Action,
    model: &mut PqueueModel,
    actual: &mut PriorityQueue<String, i32>,
) -> Result<(), TestCaseError> {
    match action {
        Action::Push(prio, item) => {
            let m_result = model.push(item.clone(), *prio);
            let a_result = actual.push(item.clone(), *prio);
            check_result(action, m_result, a_result)
        },
        Action::Remove(item) => {
            let m_result = model.remove(item);
            let a_result = actual.remove(item);
            check_result(action, m_result, a_result)
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

fn arb_push() -> impl Strategy<Value = Action> {
    (0..10i32, "[a-m]").prop_map(|(prio, item)| Action::Push(prio, item))
}

fn arb_remove() -> impl Strategy<Value = Action> {
    ("[a-m]").prop_map(|item| Action::Remove(item))
}

fn arb_action() -> impl Strategy<Value = Action> {
    prop_oneof![
        arb_push(),
        arb_remove(),
    ]
}

// Given the model, generate a possible action.
fn arb_admissable_action(model: Arc<PqueueModel>) -> BoxedStrategy<Action> {
    let m = model.clone();
    arb_action()
        .prop_filter("Precondition must apply", move |action| {
            pre_condition(&*m, action)
        })
        .boxed()
}

#[test]
fn model_basics() {
    let mut model = PqueueModel::new();
    let mut actual = PriorityQueue::new();
    let actions = vec![Action::Push(2, "c".into()), Action::Push(0, "d".into()), Action::Remove("c".into())];
    for action in actions {
        //let action = Action::Push(a as i32, a as i64);
        if pre_condition(&model, &action) {
            apply_and_check_result(&action, &mut model, &mut actual).unwrap();
        }
    }
    // model.push(2, 3)
}

proptest! {
    #[test]
    fn basic_model(actions in proptest::collection::vec(arb_action(), 1..10)) {
        let mut model = PqueueModel::new();
        let mut actual = PriorityQueue::new();
        println!("{:?}", actions);
        for action in actions {
            //let action = Action::Push(a as i32, a as i64);
            if pre_condition(&model, &action) {
                apply_and_check_result(&action, &mut model, &mut actual)?;
            }
        }
    }
}
