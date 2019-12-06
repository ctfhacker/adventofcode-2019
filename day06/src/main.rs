use std::collections::HashMap;
#[derive(Debug)]
struct Orbits<'a> {
    /// HashMap of String identifier to u32 ID for each node.
    nodes: HashMap<&'a str, u32>,

    /// HashMap of ChildId -> ParentID. This structure is needed since the only operations we care
    /// about in this graph is the path from a current node, through its parent, to the beginning
    /// of the graph.
    edges: HashMap<u32, u32>,

    /// Current number of nodes in the graph. Used as a unique ID for each node.
    num_nodes: u32
}

impl<'a> Orbits<'a> {
    pub fn new(input: &str) -> Orbits {
        let mut nodes = HashMap::new();
        let mut edges: HashMap<u32, u32> = HashMap::new();
        let mut curr_id = 0;
        // Parse the input string
        for line in input.split("\r\n") {
            if line.len() == 0 {
                continue;
            }

            // Get the string representation of the two current nodes
            let line = line.split(")").collect::<Vec<_>>();
            let parent = line[0];
            let child = line[1];

            // For each node, check to see if we already have an ID for this node, if not,
            // calculate one based on a running number
            let parent_id = match nodes.get(&parent) {
                Some(id) => *id,
                None => {
                    nodes.insert(parent, curr_id);
                    let result = curr_id;
                    curr_id += 1;
                    result
                }
            };

            // For each node, check to see if we already have an ID for this node, if not,
            // calculate one based on a running number
            let child_id = match nodes.get(&child) {
                Some(id) => *id,
                None => {
                    nodes.insert(child, curr_id);
                    let result = curr_id;
                    curr_id += 1;
                    result
                }
            };

            // Insert the child->parent edge into the hashmap since we need to traverse from child
            // to parent to find the direct and indirect orbits as well as the traversals between nodes
            edges.insert(child_id, parent_id);
        }

        Orbits {
            nodes, edges, num_nodes: curr_id
        }
    }

    /// Traverse the child->parent hashmap to calculate the path from each node to the beginning.
    /// The indirect orbits is the length of this path minus one since we don't consider the
    /// immediate parent an indirect orbit.
    pub fn indirect_orbits(&self) -> u32 {
        let mut count = 0;
        for (node_str, node_id) in self.nodes.iter() {
            let mut indirect_orbits = 0;
            let curr_parent = self.edges.get(&node_id);
            if curr_parent.is_none() {
                // Node has no parent, continue
                continue;
            }
            let mut curr_parent = curr_parent.unwrap();
            loop {
                match self.edges.get(&curr_parent) {
                    Some(parent_id) => {
                        indirect_orbits += 1;
                        curr_parent = parent_id;
                    },
                    None => break
                }
            }
            // print!("{} has {} indirect branches\n", node_str, indirect_orbits);
            count += indirect_orbits;
        }

        count
    }

    /// Direct orbits are effectively the number of nodes in the graph
    pub fn direct_orbits(&self) -> u32 {
        self.edges.keys().count() as u32
    }

    /// Finds the minimum distance between two nodes in the graph
    ///
    /// Since our graph is stored in only one direct (child -> parent), to calculate the minimum
    /// distance, the following is performed:
    /// * Find the path from the `from` node to the beginning while marking down how many steps it
    ///   took to reach each node.
    ///   i.e. for the graph A -> B -> C -> D
    ///   The result for looking at `D` is `(C: 1, B: 2, A: 3)`
    /// * Perform the same traversal for the `to` node, but at each step, check if the current node
    ///   is in the `from` nodes previously calculated path. If so, sum the current steps to the
    ///   steps found in the `from` nodes path to have the full path length.
    pub fn traverse(&self, from: &str, to: &str) -> u32 {
        let from_id = self.nodes.get(&from).expect("From key not found");
        let to_id = self.nodes.get(&to).expect("To key not found");
        let mut from_steps = HashMap::new();
        let mut curr_steps: u32 = 1;

        // Get the `from` node's ID
        let mut curr_parent = self.edges.get(&from_id).unwrap();
        loop {
            // Traverse the path backwards from the `from` node, taking note of the current number
            // of steps needed to reach the current node
            match self.edges.get(&curr_parent) {
                Some(parent_id) => {
                    from_steps.insert(parent_id, curr_steps);
                    curr_steps += 1;
                    curr_parent = parent_id;
                },
                None => break
            }
        }

        // Get the `to` node's ID
        let mut curr_parent = self.edges.get(&to_id).unwrap();
        curr_steps = 1;
        loop {
            match self.edges.get(&curr_parent) {
                // Traverse the path backwards from the `to` node.
                Some(parent_id) => {
                    // If the current node in the `to` node's path is in the `from` node's path, we
                    // have found the intersection between the paths. Sum the steps needed to reach
                    // the current node from each path
                    if from_steps.contains_key(&parent_id) {
                        return from_steps.get(&parent_id).unwrap() + curr_steps;
                    }

                    // Continue traversing backwards if the current node was not in the `from` path
                    from_steps.insert(parent_id, curr_steps);
                    curr_steps += 1;
                    curr_parent = parent_id;
                },
                None => break
            }
        }

        unreachable!();
    }
}


fn main() {
    let input = include_str!("../input");
    let orbits = Orbits::new(input);
    let indirect = orbits.indirect_orbits();
    let direct = orbits.direct_orbits();
    print!("D: {:?} + I: {}\n", direct, indirect);
    print!("Stage 1 Sum: {}\n", indirect + direct);

    print!("Stage 2: Minimum traverse YOU -> SAN: {}\n", orbits.traverse("YOU", "SAN"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        let input = "COM)B\r\nB)C\r\nC)D\r\nD)E\r\nE)F\r\nB)G\r\nG)H\r\nD)I\r\nE)J\r\nJ)K\r\nK)L";
        let orbits = Orbits::new(input);
        let indirect = orbits.indirect_orbits();
        let direct = orbits.direct_orbits();
        assert_eq!(indirect, 31);
        assert_eq!(direct, 11);
    }

    #[test]
    fn test_example_2() {
        let input = "COM)B\r\nB)C\r\nC)D\r\nD)E\r\nE)F\r\nB)G\r\nG)H\r\nD)I\r\nE)J\r\nJ)K\r\nK)L\r\nK)YOU\r\nI)SAN";
        let orbits = Orbits::new(input);
        assert_eq!(orbits.traverse("YOU", "SAN"), 4);
    }
}
