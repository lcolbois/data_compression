#![allow(dead_code)]
extern crate binary_heap_compare;
extern crate bit_vec;
use binary_heap_compare::BinaryHeapCompare;
use bit_vec::{BitVec,Iter};


enum Node {
    Leaf(u8),
    Branch(Box<Node>, Box<Node>),
}


fn scan_tree(current_node: Node, current_word: BitVec, dictionnary: &mut Vec<BitVec>) {
    match current_node {
        Node::Leaf(character) => dictionnary[character as usize] = current_word,
        Node::Branch(left_path, right_path) => {
            let mut left_path_word = current_word.clone();
            left_path_word.push(false);
            scan_tree(*left_path, left_path_word, dictionnary);

            let mut right_path_word = current_word.clone();
            right_path_word.push(true);
            scan_tree(*right_path, right_path_word, dictionnary);
        }
    }
}

fn append_bit_vec(mut a: BitVec, b: &BitVec) -> BitVec {
    a.extend(b.iter());
    a
}


fn serialize_bit_vec(x: &BitVec) -> Vec<u8> {
    let modulo = (x.len() % 8) as u8;
    let mut output = x.to_bytes();
    output.push(if modulo == 0 { 0 } else { 8 - modulo });
    return output;
}

fn unserialize_bit_vec(x: &[u8]) -> BitVec {
    let mut bits = BitVec::from_bytes(&x[..x.len() - 1]);
    let len = bits.len() - x[x.len() - 1] as usize;
    bits.truncate(len);
    bits
}

fn encode_tree(tree: &Node) -> BitVec {
    match *tree {
        Node::Leaf(character) => {
            let mut output = BitVec::new();
            output.push(true);
            append_bit_vec(output, &BitVec::from_bytes(&[character]))
        }
        Node::Branch(ref left_node, ref right_node) => {
            let mut output = BitVec::new();
            output.push(false);
            append_bit_vec(append_bit_vec(output, &encode_tree(left_node)), &encode_tree(right_node))
        }
    }
}

fn decode_tree(iter : &mut Iter) -> Node{
    match iter.next(){
        Some(val) => {
            if val {
                let mut character : u8 = 0;
                for _ in 0..8{
                    character <<= 1;
                    character |= iter.next().unwrap() as u8;
                }
                Node::Leaf(character)
            } else {
                Node::Branch(Box::new(decode_tree(iter)),Box::new(decode_tree(iter)))
            }
        }
        None => panic!()
    }

}


pub fn compression(content: Vec<u8>) -> Vec<u8> {
    let mut statistique = vec![0;256];
    for &item in content.iter() {
        statistique[item as usize] += 1;
    }

    let mut heap: BinaryHeapCompare<(u64, Node), _> =
        BinaryHeapCompare::new(|x: &(u64, Node), y: &(u64, Node)| y.0.cmp(&(x.0)));

    for index in 0..statistique.len() {
        if statistique[index] != 0 {
            heap.push((statistique[index], Node::Leaf(index as u8)));
        }
    }

    while heap.len() > 1 {
        let a = heap.pop().unwrap();
        let b = heap.pop().unwrap();
        heap.push((a.0 + b.0, Node::Branch(Box::new(a.1), Box::new(b.1))));
    }

    let mut output = encode_tree(&heap.peek().unwrap().1);

    let mut dictionnary = vec![BitVec::new();256];
    let root_word = BitVec::new();

    if let Some(final_tree) = heap.pop() {
        scan_tree(final_tree.1, root_word, &mut dictionnary);
    }

    for &item in content.iter() {
        output = append_bit_vec(output, &dictionnary[item as usize]);
    }

    serialize_bit_vec(&output)
}

fn pop_front_bit_vec(mut to_pop: BitVec) -> BitVec {
    for i in 0..to_pop.len() - 1 {
        let next = to_pop[i + 1];
        to_pop.set(i, next);
    }
    to_pop.pop();
    return to_pop;
}

fn build_tree(tree_seed: Vec<(u8, BitVec)>) -> Node {
    if tree_seed.len() == 1 {
        return Node::Leaf(tree_seed[0].0);
    } else {
        let mut left = Vec::new();
        let mut right = Vec::new();

        for item in tree_seed {
            [&mut left, &mut right][item.1[0] as usize].push((item.0, pop_front_bit_vec(item.1)));
        }

        return Node::Branch(Box::new(build_tree(left)), Box::new(build_tree(right)));
    }


}

pub fn decompression(content: Vec<u8>) -> Vec<u8> {
    // let mut dictionnary: Vec<(u8, BitVec)> = Vec::new();
    // let mut start = 0;
    // for i in 0..256 {
    //     let (character, new_start) = unserialize_bit_vec(&content[start..]);
    //     start += new_start;
    //     if character.len() > 0 {
    //         dictionnary.push((i as u8, character));
    //     }
    // }
    let input = unserialize_bit_vec(&content);
    let mut iter = input.iter();

    let tree = decode_tree(&mut iter);

    let mut output: Vec<u8> = Vec::new();
    let mut reading_head = &tree;
    for bit in iter {
        if let &Node::Branch(ref node_0, ref node_1) = reading_head {
            if bit {
                reading_head = &node_1;
            } else {
                reading_head = &node_0;
            }
            if let &Node::Leaf(character) = reading_head {
                output.push(character);
                reading_head = &tree;
            }
        }
    }
    return output;
}
