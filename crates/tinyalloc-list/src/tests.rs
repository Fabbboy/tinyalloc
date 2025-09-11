use crate::{Item, List};
use std::ptr::NonNull;

#[derive(Debug)]
struct TestNode {
    value: i32,
    next: Option<NonNull<TestNode>>,
    prev: Option<NonNull<TestNode>>,
}

impl TestNode {
    fn new(value: i32) -> Box<TestNode> {
        Box::new(TestNode {
            value,
            next: None,
            prev: None,
        })
    }
}

impl Item<TestNode> for TestNode {
    fn next(&self) -> Option<NonNull<TestNode>> {
        self.next
    }

    fn prev(&self) -> Option<NonNull<TestNode>> {
        self.prev
    }

    fn set_next(&mut self, next: Option<NonNull<TestNode>>) {
        self.next = next;
    }

    fn set_prev(&mut self, prev: Option<NonNull<TestNode>>) {
        self.prev = prev;
    }
}

fn create_node(value: i32) -> NonNull<TestNode> {
    NonNull::from(Box::leak(TestNode::new(value)))
}

unsafe fn free_node(node: NonNull<TestNode>) {
    let _ = unsafe { Box::from_raw(node.as_ptr()) };
}

#[test]
fn test_push_back_and_pop_back() {
    let mut list = List::new();

    let node1 = create_node(1);
    let node2 = create_node(2);
    let node3 = create_node(3);

    list.push(node1);
    list.push(node2);
    list.push(node3);

    assert_eq!(unsafe { node3.as_ref().value }, 3);
    assert_eq!(unsafe { node2.as_ref().value }, 2);
    assert_eq!(unsafe { node1.as_ref().value }, 1);

    let popped = list.pop().unwrap();
    assert_eq!(unsafe { popped.as_ref().value }, 3);
    unsafe {
        free_node(popped);
    }

    let popped = list.pop().unwrap();
    assert_eq!(unsafe { popped.as_ref().value }, 2);
    unsafe {
        free_node(popped);
    }

    let popped = list.pop().unwrap();
    assert_eq!(unsafe { popped.as_ref().value }, 1);
    unsafe {
        free_node(popped);
    }

    assert!(list.pop().is_none());
}

#[test]
fn test_insert_before_and_after() {
    let mut list = List::new();

    let node1 = create_node(1);
    let node2 = create_node(2);
    let node3 = create_node(3);

    list.push(node2);
    list.insert_before(node2, node1);
    list.insert_after(node2, node3);

    let values: Vec<i32> = list.iter().map(|node| node.value).collect();
    assert_eq!(values, vec![1, 2, 3]);

    unsafe {
        free_node(node1);
        free_node(node2);
        free_node(node3);
    }
}

#[test]
fn test_remove() {
    let mut list = List::new();

    let node1 = create_node(1);
    let node2 = create_node(2);
    let node3 = create_node(3);

    list.push(node1);
    list.push(node2);
    list.push(node3);

    list.remove(node2);

    let values: Vec<i32> = list.iter().map(|node| node.value).collect();
    assert_eq!(values, vec![1, 3]);

    list.remove(node1);
    let values: Vec<i32> = list.iter().map(|node| node.value).collect();
    assert_eq!(values, vec![3]);

    list.remove(node3);
    let values: Vec<i32> = list.iter().map(|node| node.value).collect();
    assert_eq!(values, vec![]);

    unsafe {
        free_node(node1);
        free_node(node2);
        free_node(node3);
    }
}

#[test]
fn test_iter() {
    let mut list = List::new();

    let node1 = create_node(10);
    let node2 = create_node(20);
    let node3 = create_node(30);

    list.push(node1);
    list.push(node2);
    list.push(node3);

    let values: Vec<i32> = list.iter().map(|node| node.value).collect();
    assert_eq!(values, vec![10, 20, 30]);

    unsafe {
        free_node(node1);
        free_node(node2);
        free_node(node3);
    }
}

#[test]
fn test_iter_mut() {
    let mut list = List::new();

    let node1 = create_node(10);
    let node2 = create_node(20);
    let node3 = create_node(30);

    list.push(node1);
    list.push(node2);
    list.push(node3);

    for node in list.iter_mut() {
        node.value *= 2;
    }

    let values: Vec<i32> = list.iter().map(|node| node.value).collect();
    assert_eq!(values, vec![20, 40, 60]);

    unsafe {
        free_node(node1);
        free_node(node2);
        free_node(node3);
    }
}

#[test]
fn test_drain() {
    let mut list = List::new();

    let node1 = create_node(100);
    let node2 = create_node(200);
    let node3 = create_node(300);

    list.push(node1);
    list.push(node2);
    list.push(node3);

    let drained: Vec<i32> = list
        .drain()
        .map(|node| unsafe {
            let value = node.as_ref().value;
            free_node(node);
            value
        })
        .collect();

    assert_eq!(drained, vec![100, 200, 300]);

    let values: Vec<i32> = list.iter().map(|node| node.value).collect();
    assert_eq!(values, vec![]);
}

#[test]
fn test_empty_list() {
    let mut list: List<TestNode> = List::new();

    assert!(list.pop().is_none());

    let values: Vec<i32> = list.iter().map(|node| node.value).collect();
    assert_eq!(values, vec![]);

    let drained: Vec<i32> = list
        .drain()
        .map(|node| unsafe {
            let value = node.as_ref().value;
            free_node(node);
            value
        })
        .collect();
    assert_eq!(drained, vec![]);
}

#[test]
fn test_single_element() {
    let mut list = List::new();
    let node = create_node(42);

    list.push(node);

    let values: Vec<i32> = list.iter().map(|node| node.value).collect();
    assert_eq!(values, vec![42]);

    let popped = list.pop().unwrap();
    assert_eq!(unsafe { popped.as_ref().value }, 42);

    let values: Vec<i32> = list.iter().map(|node| node.value).collect();
    assert_eq!(values, vec![]);

    unsafe {
        free_node(popped);
    }
}
