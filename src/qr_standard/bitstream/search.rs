use super::*;

pub(crate) mod a_star {
    #![allow(unused_variables, unreachable_code, unused_mut)]

    use super::Mode::{self, *};
    use std::collections::{HashMap, HashSet};

    // char index, char type (i.e. subset)
    // NOTE: num chars have 3 nodes, aln chars have 2, asc 1
    type NodeId = (usize, Mode);

    type Path = Vec<NodeId>;
    type Map<T> = HashMap<NodeId, T>;
    type Cost = f32;

    //     function reconstruct_path(cameFrom, current)
    fn reconstruct_path(came_from: Map<NodeId>, current: NodeId) -> Path {
        //     total_path := {current}
        let mut total_path: Path = vec![current];

        //     while current in cameFrom.Keys:
        let mut current = current;
        while let Some(&x) = came_from.get(&current) {
            //         current := cameFrom[current]
            current = x;
            //         total_path.prepend(current)
            total_path.push(current);
        }

        //     return total_path
        total_path.reverse();
        total_path
    }

    fn get(score_map: &mut Map<Cost>, node: NodeId) -> Cost {
        // as per the IEEE 754 standard, adding a finite number to infinity
        // makes infinity. meaning, this is well-behaved
        score_map.entry(node).or_insert(Cost::INFINITY).clone()
    }

    fn entry_mut<'a, T>(node_map: &'a mut Map<T>, node: &NodeId) -> &'a mut T {
        node_map.get_mut(node).expect("hashmap is empty!")
    }

    // the heuristic function
    // estimated cost of reaching the finish from here
    fn h(node: NodeId) -> Cost {
        // plan: the calculation is "go along the current mode
        // as far as possible, then switch to ascii for the remainder"
        todo!()
    }

    // distance between neighbors
    // todo: figure out how to integrate size classes
    fn d(from: NodeId, to: NodeId) -> Cost {
        if from.0 + 1 == to.0 {
            (if from.1 != to.1 {
                // add length of char count indicator
                crate::qr_standard::cc_indicator_bit_size(
                    {
                        // help!! what class do i pick?
                        2
                    },
                    to.1,
                )
            } else {
                0
            }) as Cost
                + match to.1 {
                    ASCII => 8.0,
                    AlphaNum => 5.5,
                    Numeric => 3.3,
                    Kanji => todo!(),
                }
        } else {
            Cost::INFINITY
        }
    }

    // // A* finds a path from start to goal.
    // // h is the heuristic function. h(n) estimates the cost to reach goal from node n.
    // function A_Star(start, goal, h)
    fn a_star<F>(start: NodeId, goal: NodeId, h: F) -> Option<Path>
    where
        F: Fn(NodeId) -> Cost,
    {
        //     // The set of discovered nodes that may need to be (re-)expanded.
        //     // Initially, only the start node is known.
        //     // This is usually implemented as a min-heap or priority queue rather than a hash-set.
        //     openSet := {start}
        let mut open_set: HashSet<NodeId> = HashSet::from([start]);

        //     // For node n, cameFrom[n] is the node immediately preceding it on the cheapest path from the start
        //     // to n currently known.
        //     cameFrom := an empty map
        let mut came_from: Map<NodeId> = HashMap::new();

        //     // For node n, gScore[n] is the cost of the cheapest path from start to n currently known.
        //     gScore := map with default value of Infinity
        //     gScore[start] := 0
        let mut g_score: Map<Cost> = HashMap::new();

        //     // For node n, fScore[n] := gScore[n] + h(n). fScore[n] represents our current best guess as to
        //     // how cheap a path could be from start to finish if it goes through n.
        //     fScore := map with default value of Infinity
        //     fScore[start] := h(start)
        let mut f_score: Map<Cost> = HashMap::from([(start, h(start))]);

        //     while openSet is not empty
        while !open_set.is_empty() {
            //         // This operation can occur in O(Log(N)) time if openSet is a min-heap or a priority queue
            //         current := the node in openSet having the lowest fScore[] value
            let mut current = {
                let mut nodes = open_set.iter();
                let mut best_node = *nodes.next().expect("set is empty!");
                let mut best_score = get(&mut f_score, best_node);
                for &candidate_node in nodes {
                    let try_score = get(&mut f_score, candidate_node);
                    if try_score < best_score {
                        best_node = candidate_node;
                        best_score = try_score;
                    }
                }
                best_node
            };
            //         if current = goal
            if current == goal {
                //             return reconstruct_path(cameFrom, current)
                return Some(reconstruct_path(came_from, current));
            }

            //         openSet.Remove(current)
            open_set.remove(&current);

            //         for each neighbor of current
            for neighbor in [todo!("find neighbors")] {
                //             // d(current,neighbor) is the weight of the edge from current to neighbor
                //             // tentative_gScore is the distance from start to the neighbor through current
                //             tentative_gScore := gScore[current] + d(current, neighbor)
                let tentative_g_score = get(&mut g_score, current) + todo!("implement d") as Cost;

                //             if tentative_gScore < gScore[neighbor]
                if tentative_g_score < get(&mut g_score, neighbor) {
                    //                 // This path to neighbor is better than any previous one. Record it!
                    //                 cameFrom[neighbor] := current
                    *entry_mut(&mut came_from, &neighbor) = current;
                    //                 gScore[neighbor] := tentative_gScore
                    *entry_mut(&mut g_score, &neighbor) = tentative_g_score;
                    //                 fScore[neighbor] := tentative_gScore + h(neighbor)
                    *entry_mut(&mut f_score, &neighbor) = tentative_g_score + h(neighbor);

                    //                 if neighbor not in openSet
                    //                     openSet.add(neighbor)
                    open_set.insert(
                        // note that insert() does this if clause already,
                        // so no need to implement it
                        neighbor,
                    );
                }
            }
        }

        //     // Open set is empty but goal was never reached
        //     return failure
        None
    }
}

pub(crate) mod good_star {
    #![allow(unused_variables, unused_assignments, unreachable_code, unused_mut)]

    use super::Mode::{self, *};

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
        fn cost(&self) -> Cost {
            self.0
        }
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

        // max index for a character's node list
        fn max(&self) -> usize {
            self.0.index()
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

    impl Mode {
        fn index(&self) -> usize {
            let mode = *self;
            match mode {
                ASCII => 0,
                AlphaNum => 1,
                Numeric => 2,
                Kanji => todo!(),
            }
        }
    }

    mod tests {

        use super::*;

        #[test]
        fn create_graph_nocapture() {
            let scramble = |x: usize| {
                (
                    // ((x + 1) * x)
                    if {
                        // [1, 24].contains(&x)
                        // x.count_ones().count_zeros() % 2 == 0
                        x == (((x as f32).sqrt()) as usize).pow(2)
                    } {
                        0
                        // x * 2 + 1
                    } else if {
                        //
                        (x / 10) % 2 == 0
                        // [17, 23].contains(&x)

                        //
                    } {
                        // (x / 8).pow(3)
                        1
                    } else {
                        2
                    }
                    // if x == 8 { 2 } else { 0 }
                    // if x.count_ones() % 2 == 0 {
                    //     (x % 5) + (x % 7)
                    // } else {
                    //     (x * x + 1) << (x / 2)
                    // }
                ) % 3
            };

            let mode_vec = (0..50).map(|x| Mode::LIST[scramble(x)]).collect::<Vec<_>>();

            // println!("modes:\n{:?}", mode_vec);
            for class in 0..3 {
                println!(
                    "class {}\n{:?}",
                    class,
                    crate::qr_standard::MODE_ECONOMY[class as usize]
                );
                for mode in Mode::LIST {
                    println!(
                        "-> {:?}: {}",
                        mode,
                        crate::qr_standard::cc_indicator_bit_size(class, mode)
                    );
                }

                let a = {
                    if true {
                        create_graph(&mode_vec, class)
                    } else {
                        let mut a = vec![];
                        for i in 0..5 {
                            a.push(CharNodes(
                                Mode::LIST[(i / 3) % 2 + 1],
                                [TaggedNode(i as u32, Some(Mode::LIST[(i / 3) % 2])); 3],
                            ))
                        }
                        a
                    }
                };
                println!();
                let path = optimal_path(&a);
                // println!("modes:\n{:?}", mode_vec);
                // println!("path:\n{:?}", path);
                println!();
                for (i, (&m, &p)) in mode_vec.iter().zip(path.iter()).enumerate() {
                    let m = format!("{:?}", m);
                    let p = format!("{:?}", p);
                    println!("{:2}: {:8} → {:8}", i, m, p);
                }
                println!();
                helpy_print_graph(&a);
                println!("\n");
            }
        }

        fn helpy_print_graph(graph: &Graph) {
            for (k, &CharNodes(mode, data)) in graph.iter().enumerate() {
                println!("{}: {:10?} ---", k, mode);
                for (i, &k) in data.iter().enumerate() {
                    if k < TaggedNode::default() {
                        let to_mode = Mode::LIST[i];
                        let mode_name = if let Some(mode) = k.1 {
                            format!("{:?}", mode)
                        } else {
                            "none :(".to_string()
                        };
                        println!(
                            "{:?} -> {:4} (from {})",
                            to_mode,
                            ((k.0 as f32) / 6.0).round() as i32,
                            mode_name,
                        );
                    }
                }
                println!("---");
            }
        }
    }
}
