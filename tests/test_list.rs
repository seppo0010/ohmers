#[macro_use(model, create, len, push_back, push_front, pop_back, pop_front,
        first, last, try_range, try_iter, contains, remove)] extern crate ohmers;
extern crate redis;
extern crate rustc_serialize;

use ohmers::{Ohmer, List};
use rustc_serialize::Encodable;

model!(derive { Clone } Task {
        payload: String = "".to_string();
        });

model!(derive { Clone } Queue {
        name: String = "".to_string();
        tasks: List<Task> = List::new();
        });

#[test]
fn test_list() {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();

    let q1 = create!(Queue {
            name: "q1".to_string(),
            }, &client).unwrap();
    let q2 = create!(Queue {
            name: "q2".to_string(),
            }, &client).unwrap();

    let t1 = create!(Task {
            payload: "t1".to_string(),
            }, &client).unwrap();
    let t2 = create!(Task {
            payload: "t2".to_string(),
            }, &client).unwrap();
    let t3 = create!(Task {
            payload: "t3".to_string(),
            }, &client).unwrap();
    let t4 = create!(Task {
            payload: "t4".to_string(),
            }, &client).unwrap();

    q1.tasks.push_back("tasks", &q1, &t1, &client).unwrap();
    assert_eq!(q1.tasks.len("tasks", &q1, &client).unwrap(), 1);
    q1.tasks.push_back("tasks", &q1, &t2, &client).unwrap();
    assert_eq!(q1.tasks.len("tasks", &q1, &client).unwrap(), 2);
    q1.tasks.push_back("tasks", &q1, &t3, &client).unwrap();
    assert_eq!(q1.tasks.len("tasks", &q1, &client).unwrap(), 3);

    q2.tasks.push_back("tasks", &q2, &t1, &client).unwrap();
    q2.tasks.push_back("tasks", &q2, &t2, &client).unwrap();
    q2.tasks.push_back("tasks", &q2, &t3, &client).unwrap();

    assert_eq!(q1.tasks.pop_back("tasks", &q1, &client).unwrap(), Some(t3.clone()));
    assert_eq!(q1.tasks.len("tasks", &q1, &client).unwrap(), 2);
    assert_eq!(q1.tasks.pop_front("tasks", &q1, &client).unwrap(), Some(t1.clone()));
    q1.tasks.push_front("tasks", &q1, &t1, &client).unwrap();
    assert_eq!(q1.tasks.last("tasks", &q1, &client).unwrap(), Some(t2.clone()));
    assert_eq!(q1.tasks.pop_back("tasks", &q1, &client).unwrap(), Some(t2.clone()));
    assert_eq!(q1.tasks.first("tasks", &q1, &client).unwrap(), Some(t1.clone()));
    assert_eq!(q1.tasks.pop_front("tasks", &q1, &client).unwrap(), Some(t1.clone()));
    assert_eq!(q1.tasks.first("tasks", &q1, &client).unwrap(), None);
    assert_eq!(q1.tasks.last("tasks", &q1, &client).unwrap(), None);
    assert_eq!(q1.tasks.pop_front("tasks", &q1, &client).unwrap(), None);
    assert_eq!(q1.tasks.pop_back("tasks", &q1, &client).unwrap(), None);
    assert_eq!(q1.tasks.len("tasks", &q1, &client).unwrap(), 0);

    assert_eq!(
            q2.tasks.try_range("tasks", &q2, 0, 1, &client).unwrap().collect::<Vec<_>>(),
            vec![
                t1.clone(),
                t2.clone(),
            ]
            );

    assert_eq!(
            q2.tasks.try_iter("tasks", &q2, &client).unwrap().collect::<Vec<_>>(),
            vec![
                t1.clone(),
                t2.clone(),
                t3.clone(),
            ]
            );

    assert!(q2.tasks.contains("tasks", &q2, &t1, &client).unwrap());
    assert!(q2.tasks.contains("tasks", &q2, &t2, &client).unwrap());
    assert!(q2.tasks.contains("tasks", &q2, &t3, &client).unwrap());
    assert!(!q2.tasks.contains("tasks", &q2, &t4, &client).unwrap());

    q2.tasks.push_back("tasks", &q2, &t1, &client).unwrap();
    assert_eq!(q2.tasks.remove("tasks", &q2, &t1, &client).unwrap(), 2);
    assert_eq!(q2.tasks.remove("tasks", &q2, &t1, &client).unwrap(), 0);
    assert_eq!(q2.tasks.remove("tasks", &q2, &t2, &client).unwrap(), 1);
    assert_eq!(q2.tasks.len("tasks", &q2, &client).unwrap(), 1);
    assert_eq!(q2.tasks.remove("tasks", &q2, &t3, &client).unwrap(), 1);
    assert_eq!(q2.tasks.len("tasks", &q2, &client).unwrap(), 0);
    assert_eq!(q2.tasks.remove("tasks", &q2, &t3, &client).unwrap(), 0);
}

#[test]
fn test_list_macros() {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();

    let q1 = create!(Queue {
            name: "q1".to_string(),
            }, &client).unwrap();
    let q2 = create!(Queue {
            name: "q2".to_string(),
            }, &client).unwrap();

    let t1 = create!(Task {
            payload: "t1".to_string(),
            }, &client).unwrap();
    let t2 = create!(Task {
            payload: "t2".to_string(),
            }, &client).unwrap();
    let t3 = create!(Task {
            payload: "t3".to_string(),
            }, &client).unwrap();
    let t4 = create!(Task {
            payload: "t4".to_string(),
            }, &client).unwrap();

    push_back!(q1.tasks, t1, client).unwrap();
    assert_eq!(len!(q1.tasks, &client).unwrap(), 1);
    push_back!(q1.tasks, t2, client).unwrap();
    assert_eq!(len!(q1.tasks, &client).unwrap(), 2);
    push_back!(q1.tasks, t3, client).unwrap();
    assert_eq!(len!(q1.tasks, &client).unwrap(), 3);

    push_back!(q2.tasks, t1, client).unwrap();
    push_back!(q2.tasks, t2, client).unwrap();
    push_back!(q2.tasks, t3, client).unwrap();

    assert_eq!(pop_back!(q1.tasks, client).unwrap(), Some(t3.clone()));
    assert_eq!(len!(q1.tasks, &client).unwrap(), 2);
    assert_eq!(pop_front!(q1.tasks, client).unwrap(), Some(t1.clone()));
    push_front!(q1.tasks, t1, client).unwrap();
    assert_eq!(last!(q1.tasks, client).unwrap(), Some(t2.clone()));
    assert_eq!(pop_back!(q1.tasks, client).unwrap(), Some(t2.clone()));
    assert_eq!(first!(q1.tasks, client).unwrap(), Some(t1.clone()));
    assert_eq!(pop_front!(q1.tasks, client).unwrap(), Some(t1.clone()));
    assert_eq!(first!(q1.tasks, client).unwrap(), None);
    assert_eq!(last!(q1.tasks, client).unwrap(), None);
    assert_eq!(pop_front!(q1.tasks, client).unwrap(), None);
    assert_eq!(pop_back!(q1.tasks, client).unwrap(), None);
    assert_eq!(len!(q1.tasks, &client).unwrap(), 0);

    assert_eq!(
            try_range!(q2.tasks[0 => 1], client).unwrap().collect::<Vec<_>>(),
            vec![
                t1.clone(),
                t2.clone(),
            ]
            );

    assert_eq!(
            try_iter!(q2.tasks, client).unwrap().collect::<Vec<_>>(),
            vec![
                t1.clone(),
                t2.clone(),
                t3.clone(),
            ]
            );

    assert!(contains!(q2.tasks, t1, client).unwrap());
    assert!(contains!(q2.tasks, t2, client).unwrap());
    assert!(contains!(q2.tasks, t3, client).unwrap());
    assert!(!contains!(q2.tasks, t4, client).unwrap());

    push_back!(q2.tasks, t1, client).unwrap();
    assert_eq!(remove!(q2.tasks, t1, &client).unwrap(), 2);
    assert_eq!(remove!(q2.tasks, t1, &client).unwrap(), 0);
    assert_eq!(remove!(q2.tasks, t2, &client).unwrap(), 1);
    assert_eq!(len!(q2.tasks, &client).unwrap(), 1);
    assert_eq!(remove!(q2.tasks, t3, &client).unwrap(), 1);
    assert_eq!(len!(q2.tasks, &client).unwrap(), 0);
    assert_eq!(remove!(q2.tasks, t3, &client).unwrap(), 0);
}
