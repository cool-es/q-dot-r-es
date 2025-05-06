// a simpler search algorithm than a*:
// compute the "g score" of every node,
// backtrack and pick the lowest value.
//
// if two characters are of the same type,
// there is no reason to switch modes.
//
// any message of length n has at most
// 6n-6 edges (alternating between aln-asc).
use super::{Mode, char_status, tables};

/// The cheapest known way to reach a character.
type Cost = u32;

/// A node in a graph, with extra information.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct TaggedNode(
    /// The cost of the cheapest way to
    /// get here.
    Cost,
    /// The ID of the previous node along
    /// the cheapest known way to get here.
    Option<Mode>,
);

impl TaggedNode {
    fn pointer(&self) -> Option<Mode> {
        self.1
    }
}

impl Default for TaggedNode {
    fn default() -> Self {
        TaggedNode(Cost::MAX, None)
    }
}

impl PartialOrd for TaggedNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

/// The nodes associated with a character.
///
/// Each character is given a node for every subset it is part of.
#[derive(Clone, Copy)]
struct CharNodes(Mode, [TaggedNode; 3]);

impl CharNodes {
    /// Access a certain node for a given character.
    fn get(&self, category: Mode) -> Option<TaggedNode> {
        if !self.has(category) {
            None
        } else {
            Some(self.1[category.index()])
        }
    }

    /// A mutable reference to a certain node.
    fn get_mut(&mut self, category: Mode) -> Option<&mut TaggedNode> {
        if !self.has(category) {
            None
        } else {
            Some(&mut self.1[category.index()])
        }
    }

    /// Replace a cost value in a node, if the new value is smaller.
    fn set_min(&mut self, category: Mode, value: TaggedNode) {
        if let Some(v) = self.get(category) {
            if value < v {
                self.1[category.index()] = value;
            }
        }
    }

    /// Does this character have a node of this type?
    fn has(&self, category: Mode) -> bool {
        // this optimization doesn't hold for kanji...
        category >= self.0
        // category.index() <= self.max()
    }

    /// Propagate the lowest-cost route alternatives from
    /// the previous character.
    ///
    /// This function forms the backbone of the search algorithm.
    fn score_from_predecessor(&mut self, from: &Self, class: u8) {
        // check each node we're going from
        for from_mode in Mode::LIST.into_iter() {
            // if that node exists,
            if let Some(TaggedNode(from_score, _)) = from.get(from_mode) {
                // check each node we're moving towards
                for to_mode in Mode::LIST.into_iter() {
                    // and if THAT node exists,
                    if let Some(TaggedNode(..)) = self.get(to_mode) {
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

    /// The type of the node with the lowest cost.
    fn cheapest_mode(&self) -> Mode {
        use Mode::{AlphaNum, Numeric, ASCII};

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

/// The nodes corresponding to the full message.
type Graph = Vec<CharNodes>;

/// Calculate the added cost of getting from one node to another.
///
/// We approximate the cost of alphanumerics and
/// numerics as their average bits per character:
/// 5.5 (11 / 2) and 3.33... (10 / 3) respectively.
/// We multiply this by 6 to get an integer again,
/// so numeric characters are awarded 20 points,
/// alphanumerics 33, and ASCII characters 48.
///
/// Switching to a new mode incurs an overhead (mode switch
/// marker and character count indicator), which is only
/// added if the `same_subset` flag is set to `false`.
fn edge_weight(to_mode: Mode, same_subset: bool, class: u8) -> Cost {
    (if !same_subset {
        // we multiply by 6 to get rid of decimals
        // the 4 is the size of the mode indicator
        6 * (4 + tables::cc_indicator_bit_size(class, to_mode)) as Cost
    } else {
        0
    }) + match to_mode {
        // 6 * 8
        Mode::ASCII => 48,
        // 6 * 11/2
        Mode::AlphaNum => 33,
        // 6 * 10/3
        Mode::Numeric => 20,
    }
}

/// Create a graph of nodes, along with their respective costs and pointers.
fn create_graph(mode_vec: &[Mode], class: u8) -> Graph {
    let mut mode_iter = mode_vec.iter();

    // first character is a special case - the "same subset" parameter
    // is false for all modes
    let mut current_nodes = CharNodes(
        *mode_iter.next().expect("mode vector is empty"),
        [TaggedNode::default(); 3],
    );

    for mode in Mode::LIST {
        match current_nodes.get_mut(mode) {
            Some(node) => {
                *node = TaggedNode(edge_weight(mode, false, class), None);
            }
            None => break,
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

/// Retrace the optimal path back through a graph.
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
        match character.get(current_mode).unwrap().pointer() {
            Some(mode) => {
                current_mode = mode;
                output.push_front(current_mode);
            }
            None => break,
        }
    }

    Vec::from(output)
}

/// Optimize
pub fn optimize_mode(string: &String, class: u8) -> Vec<(Mode, String)> {
    let char_to_mode = |x| char_status(x).unwrap_or(Mode::ASCII);

    if string.is_empty() {
        return vec![];
    }
    if string.chars().count() == 1 {
        let mode = char_to_mode(string.bytes().next().unwrap() as char);
        return vec![(mode, string.to_string())];
    }

    let mode_vec = string.chars().map(char_to_mode).collect::<Vec<_>>();

    let good_vec = optimal_path(&create_graph(&mode_vec, class));

    let mut zip = good_vec.into_iter().zip(string.chars());
    let (mut current_mode, chr) = zip.next().unwrap();
    let mut push_string = String::from(chr);

    let mut output = vec![];

    for (mode, chr) in zip {
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
            Self::ASCII => 0,
            Self::AlphaNum => 1,
            Self::Numeric => 2,
        }
    }
}
