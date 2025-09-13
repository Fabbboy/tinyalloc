use crate::{HasLink, Link, List};
use std::ptr::NonNull;

#[derive(Debug, Default)]
struct TestNode {
    value: i32,
    link: Link<TestNode>,
}

impl TestNode {
    fn new(value: i32) -> Box<TestNode> {
        Box::new(TestNode {
            value,
            link: Link::default(),
        })
    }
}

impl HasLink<TestNode> for TestNode {
    fn link(&self) -> &Link<TestNode> {
        &self.link
    }

    fn link_mut(&mut self) -> &mut Link<TestNode> {
        &mut self.link
    }
}

fn create_node(value: i32) -> NonNull<TestNode> {
    NonNull::from(Box::leak(TestNode::new(value)))
}

unsafe fn free_node(node: NonNull<TestNode>) {
    let _ = unsafe { Box::from_raw(node.as_ptr()) };
}

#[test]
fn test_push_and_pop() {
    let mut list: List<TestNode> = List::new();

    let node1 = create_node(1);
    let node2 = create_node(2);
    let node3 = create_node(3);

    list.push(node1);
    list.push(node2);
    list.push(node3);

    let popped = list.pop().unwrap();
    assert_eq!(unsafe { popped.as_ref().value }, 3);
    unsafe { free_node(popped); }

    let popped = list.pop().unwrap();
    assert_eq!(unsafe { popped.as_ref().value }, 2);
    unsafe { free_node(popped); }

    let popped = list.pop().unwrap();
    assert_eq!(unsafe { popped.as_ref().value }, 1);
    unsafe { free_node(popped); }

    assert!(list.pop().is_none());
}

#[test]
fn test_iter() {
    let mut list: List<TestNode> = List::new();

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
fn test_remove() {
    let mut list: List<TestNode> = List::new();

    let node1 = create_node(1);
    let node2 = create_node(2);
    let node3 = create_node(3);

    list.push(node1);
    list.push(node2);
    list.push(node3);

    list.remove(node2);

    let values: Vec<i32> = list.iter().map(|node| node.value).collect();
    assert_eq!(values, vec![1, 3]);

    unsafe {
        free_node(node1);
        free_node(node2);
        free_node(node3);
    }
}

#[test]
fn test_insert_operations() {
    let mut list: List<TestNode> = List::new();

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
fn test_single_list_membership_only() {
    let mut list1: List<TestNode> = List::new();
    let mut list2: List<TestNode> = List::new();
    
    let node = create_node(42);
    
    list1.push(node);
    assert!(list1.contains(node));
    assert!(!list2.contains(node));
    
    list2.push(node);
    
    assert!(!list1.contains(node));
    assert!(list2.contains(node));
    
    let removed = list2.pop().unwrap();
    assert_eq!(unsafe { removed.as_ref().value }, 42);
    
    unsafe {
        free_node(removed);
    }
}

