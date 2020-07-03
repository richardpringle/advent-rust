use std::{
    collections::{hash_set, HashMap, HashSet},
    error::Error,
    fs,
    iter::Iterator,
};


type Relationship = (String, String);

// This solution is definitely a brute-force solution!
// I rely heavily on HashSets to get a unique list of nodes
// as well as comparing pathways. I'm sure there's some graph algorith to properly find
// the shortest path for part-2.
// I do however, love how easy it was to travers the HashMap with an iterator!

#[derive(Debug)]
struct RelationshipIter<'a> {
    map: HashMap<&'a str, &'a str>,
    nodes: hash_set::IntoIter<&'a str>,
    next: Option<&'a str>,
}

impl<'a> RelationshipIter<'a> {
    fn from_list(list: &'a [Relationship]) -> Self {
        let map: HashMap<&str, &str> = list
            .iter()
            .map(|(parent, child)| (child.as_str(), parent.as_str()))
            .collect();
        let mut nodes = map.keys().copied().collect::<HashSet<_>>().into_iter();

        let next = nodes.next();

        RelationshipIter { map, next, nodes }
    }

    fn with_single_traversal_from_node(list: &'a [Relationship], node: &'a str) -> Self {
        let map: HashMap<_, _> = list
            .iter()
            .map(|(parent, child)| (child.as_str(), parent.as_str()))
            .collect();
        let nodes = HashSet::new().into_iter();
        let next = Some(node);

        RelationshipIter { map, next, nodes }
    }
}

impl<'a> Iterator for RelationshipIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self
            .next
            .filter(|node| node != &"COM")
            .or_else(|| self.nodes.next()) // I really like these helper methods
            .take();

        self.next = next.and_then(|node| self.map.get(node)).copied();

        next
    }
}

fn solve_1(input: &[Relationship]) -> usize {
    RelationshipIter::from_list(input).count()
}

fn resolve_none_on_found<'a>(
    set: HashSet<&'a str>,
) -> impl FnMut(&mut bool, &'a str) -> Option<&'a str> {
    move |found, node| {
        if *found {
            None
        } else {
            *found = set.contains(node);
            Some(node)
        }
    }
}

fn solve_2(input: &[Relationship]) -> usize {
    let (me, san) = ("YOU", "SAN");
    let my_visited_nodes: HashSet<_> =
        RelationshipIter::with_single_traversal_from_node(input, &me).collect();
    let (san_count, common_node) = RelationshipIter::with_single_traversal_from_node(input, &san)
        .scan(false, resolve_none_on_found(my_visited_nodes))
        .enumerate()
        .last()
        .unwrap();
    let (my_count, _) = RelationshipIter::with_single_traversal_from_node(input, &me)
        .take_while(|node| node != &common_node)
        .enumerate()
        .last()
        .unwrap();

    san_count + my_count - 1
}

fn main() {
    let input = get_input().unwrap_or_else(|err| {
        eprintln!("{}", err);
        std::process::exit(1);
    });

    println!("first solution: {:?}", solve_1(input.as_ref()));
    println!("second solution: {:?}", solve_2(input.as_ref()));
}

fn get_input() -> Result<Vec<Relationship>, Box<dyn Error>> {
    Ok(fs::read_to_string("input.txt")?
        .lines()
        .map(|line| line.split(')').collect::<Vec<_>>())
        .map(|sides| (sides[0].to_string(), sides[1].to_string()))
        .collect())
}
