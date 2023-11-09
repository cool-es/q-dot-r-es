#![allow(unused_variables, unused_assignments, unreachable_code, unused_mut)]

use super::*;

// a simpler search algorithm than a*
// compute the "g score" of every node,
// backtrack and pick the lowest value

// if two characters are of the same type,
// there is no reason to switch modes

// any message of length n has at most
// 6n-6 edges (alternating between aln-asc)

// the cheapest known way to reach a character
// we approximate the cost of a single aln/num
// char as 11/2 and 10/3 respectively, but if we
// multiply it by 6 it makes an integer
// 10/3 -> 20
// 11/2 -> 33
// 8 -> 48

type Cost = u32;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct TaggedNode(Cost, Option<Mode>);

impl TaggedNode {
    fn pointer(&self) -> Option<Mode> {
        self.1
    }
}

impl Default for TaggedNode {
    fn default() -> Self {
        TaggedNode(u32::MAX, None)
    }
}

impl PartialOrd for TaggedNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

// the 1, 2 or 3 nodes associated with
// a character
#[derive(Clone, Copy)]
struct CharNodes(Mode, [TaggedNode; 3]);

impl CharNodes {
    fn get(&self, category: Mode) -> Option<TaggedNode> {
        if !self.has(category) {
            None
        } else {
            Some(self.1[category.index()])
        }
    }

    fn get_mut(&mut self, category: Mode) -> Option<&mut TaggedNode> {
        if !self.has(category) {
            None
        } else {
            Some(&mut self.1[category.index()])
        }
    }

    fn set_min(&mut self, category: Mode, value: TaggedNode) {
        if let Some(v) = self.get(category) {
            if value < v {
                self.1[category.index()] = value;
            }
        }
    }

    fn has(&self, category: Mode) -> bool {
        // this optimization doesn't hold for kanji...
        category >= self.0
        // category.index() <= self.max()
    }

    // propagate best score + edge weight
    fn score_from_predecessor(&mut self, from: &Self, class: u8) {
        // check each node we're going from
        for from_mode in Mode::LIST.into_iter() {
            // if that node exists,
            if let Some(TaggedNode(from_score, _)) = from.get(from_mode) {
                // check each node we're moving towards
                for to_mode in Mode::LIST.into_iter() {
                    // and if THAT node exists,
                    if let Some(TaggedNode(current_to_score, _)) = self.get(to_mode) {
                        // are we moving between two nodes of the same type?
                        let same_subset = from_mode == to_mode;

                        // calculate the value of the node we're moving towards
                        let tentative_to_score =
                            from_score + edge_weight(to_mode, same_subset, class);

                        // if the score is lower than what's already there,
                        // i.e. we're on a more optimal path, replace it
                        self.set_min(to_mode, TaggedNode(tentative_to_score, Some(from_mode)));
                    } else {
                        break;
                    }
                }
            } else {
                break;
            }
        }
    }

    fn cheapest_mode(&self) -> Mode {
        let mut cheapest_mode = ASCII;
        let mut lowest_cost = self.get(ASCII).unwrap();
        for category in [AlphaNum, Numeric] {
            if let Some(node) = self.get(category) {
                if node < lowest_cost {
                    (cheapest_mode, lowest_cost) = (category, node);
                }
            }
        }
        cheapest_mode
    }
}

// the nodes corresponding to the full message
type Graph = Vec<CharNodes>;

// averaged distance between neighbors
fn edge_weight(to_mode: Mode, same_subset: bool, class: u8) -> Cost {
    (if !same_subset {
        // we multiply by 6 to get rid of decimals
        // the 4 is the size of the mode indicator
        6 * (4 + crate::qr_standard::cc_indicator_bit_size(class, to_mode)) as Cost
    } else {
        0
    }) + match to_mode {
        // 6 * 8
        ASCII => 48,
        // 6 * 11/2
        AlphaNum => 33,
        // 6 * 10/3
        Numeric => 20,
        Kanji => todo!(),
    }
}

// creates a graph of nodes along with their "g scores"
fn create_graph(mode_vec: &Vec<Mode>, class: u8) -> Graph {
    let mut mode_iter = mode_vec.iter();

    // first character is a special case - the "same subset" parameter
    // is false for all modes
    let mut current_nodes = CharNodes(
        *mode_iter.next().expect("mode vector is empty"),
        [TaggedNode::default(); 3],
    );

    for mode in Mode::LIST {
        if let Some(node) = current_nodes.get_mut(mode) {
            *node = TaggedNode(edge_weight(mode, false, class), None);
        } else {
            break;
        }
    }

    let mut output: Graph = vec![current_nodes];

    // remainder of graph
    let mut previous_nodes: CharNodes;
    for &char_mode in mode_iter {
        (previous_nodes, current_nodes) = (
            current_nodes,
            CharNodes(char_mode, [TaggedNode::default(); 3]),
        );
        current_nodes.score_from_predecessor(&previous_nodes, class);
        output.push(current_nodes);
    }
    output
}

fn optimal_path(graph: &Graph) -> Vec<Mode> {
    if graph.is_empty() {
        return vec![];
    }

    let mut graph_iter = graph.iter();
    let mut output = std::collections::VecDeque::new();

    let last_char = graph_iter.next_back().unwrap();
    let mut current_mode = last_char.cheapest_mode();
    output.push_front(current_mode);
    current_mode = last_char.get(current_mode).unwrap().pointer().unwrap();
    output.push_front(current_mode);

    while let Some(&character) = graph_iter.next_back() {
        if let Some(mode) = character.get(current_mode).unwrap().pointer() {
            current_mode = mode;
            output.push_front(current_mode);
        } else {
            break;
        }
    }

    Vec::from(output)
}

pub(crate) fn optimize_mode(string: String, class: u8) -> Vec<(Mode, String)> {
    if string.is_empty() {
        todo!()
    }

    let mode_vec = string.chars().map(|x| char_status(x).unwrap()).collect();

    let good_vec = optimal_path(&create_graph(&mode_vec, class));

    let mut zip = good_vec.into_iter().zip(string.chars());
    let (mut current_mode, chr) = zip.next().unwrap();
    let mut push_string = String::from(chr);

    let mut output = vec![];

    // println!("{}: {:?}", chr, current_mode);
    for (mode, chr) in zip {
        // println!("{}: {:?}", chr, mode);
        if mode == current_mode {
            push_string.push(chr);
        } else {
            output.push((current_mode, push_string.clone()));
            current_mode = mode;
            push_string = String::from(chr);
        }
    }
    output.push((current_mode, push_string.clone()));

    output
}

impl Mode {
    fn index(self) -> usize {
        match self {
            ASCII => 0,
            AlphaNum => 1,
            Numeric => 2,
            Kanji => todo!(),
        }
    }
}
