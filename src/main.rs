use std::path::Path;
use mcmf::{GraphBuilder, Vertex, Cost, Capacity};

struct Case {
    pub name: &'static str,
    pub weight: usize,
}

const NUM_CASES: usize = 9;
const CASES: [Case; NUM_CASES] = [
    Case { name: "UNCCD", weight: 12, },
    Case { name: "Syngenta", weight: 14, },
    Case { name: "BellFoodGroup", weight: 14, },
    Case { name: "City St.Gallen", weight: 10, },
    Case { name: "Canton St.Gallen", weight: 10, },
    Case { name: "Cisco", weight: 7, },
    Case { name: "University of St.Gallen ", weight: 12, },
    Case { name: "START Fellowship", weight: 9, },
    Case { name: "SIX", weight: 14, },
];

struct Record {
    pub team_name: String,
    pub allocation: [i32; NUM_CASES],
}

fn parse_allocation(tokens: Vec<&str>) -> Result<[i32; NUM_CASES], Box<dyn std::error::Error>> {
    let mut allocation: [i32; NUM_CASES] = [0; NUM_CASES];
    for i in 0..NUM_CASES {
        let v = tokens[i].parse()?;
        if !(v >= 1 && v <= NUM_CASES as i32) {
            return Err(format!("Points must be between 1 and {}", NUM_CASES).into())
        }
        allocation[i] = v;
    }
    // check for duplicates
    for i in 0..NUM_CASES {
        for j in i+1..NUM_CASES {
            if allocation[i] == allocation[j] {
                return Err("Cases cannot be assigned the same points".into())
            }
        }
    }
    Ok(allocation)
}

fn parse_record(line: &str) -> Record {
    let mut tokens: Vec<&str> = line.rsplitn(NUM_CASES + 1, ',').collect();
    // skip 20 to remove time stamp
    let team_name = &String::from(tokens.pop().unwrap())[20..];
    let tokens: Vec<&str> = tokens.into_iter().rev().collect();
    let allocation = match parse_allocation(tokens) {
        Ok(allocation) => allocation,
        Err(why) => panic!("[ERROR] Team \"{}\": Invalid point allocation: {}", team_name, why),
    };
    Record { team_name: String::from(team_name), allocation }
}


fn import_csv(path: &str) -> Vec<Record> {
    let path = Path::new(path);
    let file_content = match std::fs::read_to_string(&path) {
        Err(why) => panic!("Couldn't open \"{}\": {}", path.display(), why),
        Ok(file_content) => file_content,
    };
    let mut records: Vec<Record> = Vec::new();
    for line in file_content.lines().skip(1) {
        let record = parse_record(line);
        // println!("Record {{ team=\"{}\" allocation={:?} }}", record.team_name, record.allocation);
        records.push(record);
    }
    records
}

fn max_teams_per_case(num_teams: usize) -> [usize; NUM_CASES] {
    let mut total_weight = 0;
    for case in CASES {
        total_weight += case.weight;
    }
    let mut result: [usize; NUM_CASES] = [0; NUM_CASES];
    for i in 0..NUM_CASES {
        result[i] = num_teams * CASES[i].weight / total_weight + 1;
    }
    result
}


fn main() {
    let records = import_csv("input.csv");

    let mut graph = GraphBuilder::<String>::new();
    // source to teams
    for record in records.iter() {
        graph.add_edge(Vertex::Source, record.team_name.clone(), Capacity(1), Cost(0));
    }
    // teams to cases
    for record in records.iter() {
        for (i, case) in CASES.iter().enumerate() {
            graph.add_edge(record.team_name.clone(), String::from(case.name), Capacity(1), Cost(record.allocation[i]));
        }
    }
    // cases to sink
    let teams_per_case = max_teams_per_case(records.len());
    for (i, case) in CASES.iter().enumerate() {
        graph.add_edge(String::from(case.name), Vertex::Sink, Capacity(teams_per_case[i] as i32), Cost(0));
    }

    let (_cost, paths) = graph.mcmf();
    assert_eq!(records.len(), paths.len());

    for path in paths {
        let vertices = path.vertices();
        assert_eq!(vertices.len(), 4);
        let team_name = vertices[1].clone().as_option().unwrap();
        let case_name = vertices[2].clone().as_option().unwrap();
        println!("Team \"{}\" -> Case \"{}\"", team_name, case_name);
    }
}
