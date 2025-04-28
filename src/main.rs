use ndarray::{Array, Array2};
use csv::ReaderBuilder;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;

mod spam_functions;


// It wraps different types as different variants of the same type since arrays must have elements of the same type
#[derive(Debug, Clone, PartialEq)]
pub enum ColumnVal {
    One(String),
    Two(bool),
}



// It represents the data in an array format and keeps track of misc info like the total # of users and total # of spam users
#[derive(Debug)]
struct DataFrame {
    data: Array2<ColumnVal>,
    total_users: u32,
    total_spam_users: u32,
}

fn main() {

    let my_arr: Array2<ColumnVal> = read_CSV_using_reader("Youtube-Spam-Dataset.csv");
    let (my_map, users) = map_users_to_words(&my_arr); // my_map maps a user to a hashset of words they used. users is a vector of unique users in some random order
    
    let (num_spam_users, spammers) = spam_functions::find_spam(&my_arr, &users); // spammers is a vector of unique spammers
    let df = DataFrame::new(my_arr, users.len() as u32, num_spam_users); // a dataframe for all users in the dataset

    let my_graph_sim: HashMap<String, Vec<String>> = create_graph(&users, &my_map, 0.7); // maps a name to a vector of names of people whose similarity index with them is at least the threshold

    let num_graphs_all_nodes = find_num_disconnected_graphs(&my_graph_sim);
    println!("Num graphs (all nodes included), threshold 0.7: {:?}", num_graphs_all_nodes);
    
    

    // Spam only
    let spam_arr: Array2<ColumnVal> = spam_functions::spam_specific_arr("Youtube-Spam-Dataset.csv");
    let (map_of_spam, spammers2) = map_users_to_words(&spam_arr); // spammers2 is the same vector as spammers; the order might be a little different

    let my_graph_spam_only: HashMap<String, Vec<String>> = create_graph(&spammers2, &map_of_spam, 0.7);
    let num_disconnected_graphs_spam_only = find_num_disconnected_graphs(&my_graph_spam_only);
    println!("Num graphs in the spam-only network, threshold 0.7: {:?}", num_disconnected_graphs_spam_only);

    // Some quick stats
    df.quick_stats();

    // Analyzing number of disconnected graphs based on different similarity thresholds
    let thresholds = vec![0.0, 0.2, 0.5, 0.9, 1.0];
    for threshold in thresholds.iter() {
        let spam_graph: HashMap<String, Vec<String>> = create_graph(&spammers2, &map_of_spam, *threshold);
        let num_disconnected = find_num_disconnected_graphs(&spam_graph);
        println!("Num graphs in the spam-only network, threshold {:?}: {:?}", threshold, num_disconnected);
    }

    // Find the best spammers and the words they used
    let best_spammers: HashSet<String> = spam_functions::find_best_spammer(&my_graph_spam_only);
    let mut best_words: HashSet<String> = HashSet::new();
    let mut num_best = 0;

    for spammer in best_spammers.iter() {
        num_best += 1;
        for word in my_map.get(&spammer.clone()).unwrap(){
            best_words.insert(word.to_string());
        }
    }

    println!("");
    println!("There was/were {:?} best spammer(s) (who had the most similarities with other spammers). They were {:?}, and used the following words: {:?}", num_best, best_spammers, best_words);

}


impl DataFrame {
    
    // creates a new instance based on parameters
    fn new(df: Array2<ColumnVal>, num_users: u32, num_spam_users: u32) -> DataFrame {
        return DataFrame {
            data: df,
            total_users: num_users,
            total_spam_users: num_spam_users,
        }
    }

    // prints some quick stats. Read the names as needed
    fn quick_stats(&self) {
        let mut counter_spam_comments = 0;
        for colval in self.data.column(3).iter() {
            match colval {
                ColumnVal::One(_) => (),
                ColumnVal::Two(some_bool) => {
                    if *some_bool {
                        counter_spam_comments += 1;
                    }
                }
            };
        }
        println!("");
        println!("Some quick stats:");
        println!("Number of total unique users: {:?}", self.total_users);
        println!("Number of unique spam users: {:?}", self.total_spam_users);
        println!("Number of spam comments in dataset: {:?}", counter_spam_comments);
        println!("");
    }
}


// purpose: maps users to a hashset of unique words they used across all their comments
// input: an array of data
// output: a hashmap mapping a user's name to a hashset of their unique words, as well as a vector of unique users' names in some random order
// it iterates over the users' names in col 0. Also iterates over each row of data to extract the comment text and split it into words (alphanumeric chars only)
fn map_users_to_words(in_df: &Array2<ColumnVal>) -> (HashMap<String, HashSet<String>>, Vec<String>) { // in_df has the following cols: 0-user, 1-content, 2-video name, 3-class

    let mut unique_users: HashSet<String> = HashSet::new();
    let mut users_to_words: HashMap<String, HashSet<String>> = HashMap::new();
    let mut out_users: Vec<String> = Vec::new();

    let num_rows = in_df.nrows();

    for user in in_df.column(0) {
        match user {
            ColumnVal::One(name) => {
                unique_users.insert(name.clone());
                ()
            }
            ColumnVal::Two(_) => {
                println!("Should not happen, as users are not bools");
                ()
            }
        }
    }
    for unique_user in unique_users.iter() {
        out_users.push(unique_user.clone()); // create your out vector of unique users' names
    }

    for i in 0..num_rows { // iterate over each row
        let given_content_as_CV = in_df[[i, 1]].clone();
        let given_user_as_CV = in_df[[i, 0]].clone();
        let mut given_content = String::new(); // stores the comment content
        let mut given_user = String::new(); // stores the user's name

        match given_content_as_CV {
            ColumnVal::One(content) => given_content = content,
            _ => println!("Should not execute"),
        }
        match given_user_as_CV {
            ColumnVal::One(name) => given_user = name,
            _ => println!("Should not execute"),
        }

        let words = given_content.split(' '); // an iterator
        let mut tmp_hashset: HashSet<String> = HashSet::new(); // will store the unique words the user used

        for word in words {
            if word != "" {
                tmp_hashset.insert(word.chars() // turn the word into characters, filter out non-alphanumeric, turn it back into a String, make it lowercase
                .filter(|c| c.is_alphanumeric())
                .collect::<String>().to_lowercase().to_string());
            }
        }
        if users_to_words.contains_key(&given_user) { // if the key (the user) exists, just update the current hashset of words
            let mut original_map = users_to_words.get(&given_user).unwrap().clone();
            original_map.extend(tmp_hashset);
            users_to_words.insert(given_user, original_map);
        } else { // if the key does not yet exist, insert the new key-value pair
            users_to_words.insert(given_user.to_string(), tmp_hashset);
        }

    }
    return (users_to_words, out_users);
}


// purpose: find the similarity index for two users. This is represented by the formula (number of unique words
// they shared across all their comments) / (number of unique words either person1 or person2 used).
// input: person1's name, person2's name, a hashmap mapping a name to a hashset of words they used
// output: an Option: Some(the index) or None, when either person doesn't exist in the hashmap
// it iterates over each person's set of words, finds the same ones and counts those, and divides that by the total number of unique words used
fn find_similarities(person1: String, person2: String, dict: &HashMap<String, HashSet<String>>) -> Option<f64> {
    let mut shared_set: HashSet<String> = HashSet::new();
    let mut total_set: HashSet<String> = HashSet::new();

    if dict.contains_key(&person1) == false || dict.contains_key(&person2) == false {
        println!("One of these people are not found in hashmap");
        return None;
    }

    let person1_set = dict.get(&person1).expect("Person not found");
    let person2_set = dict.get(&person2).expect("Person not found");

    for item in person1_set.iter() { // iterate over person1's words used
        if person2_set.contains(item) { // catch shared words
            shared_set.insert(item.clone());
        }
        total_set.insert(item.clone()); // add all person1's words to total
    }
    for item2 in person2_set.iter() { // add all person2's words to total
        total_set.insert(item2.clone());
    }

    let shared_len: f64 = shared_set.len() as f64;
    let total_len: f64 = total_set.len() as f64;

    return Some(shared_len / total_len);

}



// purpose: create the graph based on the users, the words each used, and a threshold for similarity
// input: a vector of unique users, a hashmap mapping a user to the words they used, and a threshold for similarity
// output: a hashmap mapping a user to a vector of their neighbors in the graph
// iterate over each pair of unique users (no repeats) and calculate the similarity index for the user-user pair. If the sim index
// is > threshold, the users will be neighbors of each other in the graph
fn create_graph(users: &Vec<String>, map: &HashMap<String, HashSet<String>>, threshold: f64) -> HashMap<String, Vec<String>> {
    
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();
    
    for user in users.iter() {
        graph.insert(user.to_string(), vec![]);
    }

    for i in 0..users.len() {
        for j in i..users.len() { // no repeats. only consider users i and on
            if i != j { // don't compare a user to itself
                let user1 = &users[i];
                let user2 = &users[j];
                let sim_index = find_similarities(user1.to_string(), user2.to_string(), &map).unwrap(); // find_similarities returns an Option
                if sim_index >= threshold {
                    let mut tmp_vec = graph.get(&users[i]).unwrap().clone(); // get the current vector
                    tmp_vec.push(users[j].to_string()); // append to current vector
                    graph.insert(users[i].clone(), tmp_vec); // update the hashmap

                    // and do the same thing for user[j]
                    let mut tmp_vec2 = graph.get(&users[j]).unwrap().clone(); // get the current vector
                    tmp_vec2.push(users[i].to_string()); // append to current vector
                    graph.insert(users[j].clone(), tmp_vec2); // update the hashmap
                }
            }
        }
    }

    return graph;
}


// purpose: find the number of disconnected subgraphs in the entire graph
// input: a graph mapping users to their neighbors
// output: a u32 number of disconnected subgraphs
// use a while loop and consider each node one at a time. Traverse the graph and when there are no more neighbors to traverse, the subgraph has been traversed
// and we can randomly jump to another node that hasn't been seen before
fn find_num_disconnected_graphs(graph: &HashMap<String, Vec<String>>) -> u32 {
    let mut vec_keys: Vec<String> = Vec::new();
    let mut counter: u32 = 0; // this counts the number of disconnected graphs


    for key in graph.keys() {
        vec_keys.push(key.to_string()); // create a vector of keys to represent possible places to check next
    }

    let mut seen: HashSet<String> = HashSet::new();

    while vec_keys.len() != 0 {
        let start = vec_keys.pop().unwrap(); // pick a start node
        if seen.contains(&start) == false { // proceed if it's a node that we haven't seen before
            counter += 1; // this is indeed a new disconnected subgraph, so increment
            let mut queue = VecDeque::from([start.clone()]);
            seen.insert(start);

            while queue.len() != 0 { // while we have other places to explore
                let mut current = format!("Dummy value");
                if let Some(node) = queue.pop_front() {
                    current = node;
                }
                
                match graph.get(&current) { // check the neighbors of current
                    Some(vec_of_neighbors) => {
                        for neighbor in vec_of_neighbors {
                            if seen.contains(neighbor) == false {
                                seen.insert(neighbor.to_string()); // mark the neighbor as seen
                                queue.push_back(neighbor.to_string()); // mark the neighbor as some place to check out
                            }
                        }
                    },
                    None => {
                        println!("This should not execute!");
                        ()
                    }
                }
            }
        }


    }

    return counter;

}


// purpose: put the relevant CSV data into an array
// input: a path with the CSV name
// output: an Array2 of ColumnVals representing the relevant data
// iterate over the lines of the CSV, split by commas but treat things in double quotes as single entries. Gets only the cols of interest
fn read_CSV_using_reader(path: &str) -> Array2<ColumnVal> {
    let mut rdr = csv::ReaderBuilder::new()
    .has_headers(true)
    .delimiter(b',')
    .double_quote(true)
    .escape(Some(b'\\'))
    .flexible(false)
    .from_path(path).unwrap();

    let headers = rdr.headers();

    let mut counter = 0; // this counts number of rows

    let mut giant_vec: Vec<ColumnVal> = Vec::new(); // the out array will be created using this and reshaping

    for result in rdr.records() {
        match result {
            Ok(record) => { // record is a line
                counter += 1; // we've found a new row
                for (num, item) in record.iter().enumerate() { // item is a particular cell
                    if num == 1 || num == 3 || num == 4 { // cols of interest are user, comment content, video name
                        giant_vec.push(ColumnVal::One(item.to_string()));
                    }
                    if num == 5 { // and the fourth col of interest is the classification as spam or not
                        if item == "1" {
                            giant_vec.push(ColumnVal::Two(true));
                        } else if item == "0" {
                            giant_vec.push(ColumnVal::Two(false));
                        } else {
                            println!("This should not execute");
                        }
                        
                    }
                }
            },
            Err(err) => {
                println!("error reading CSV record {}", err);
            }  
        }
    }
    
    let out_arr: Array2<ColumnVal> = Array::from_vec(giant_vec).into_shape((counter, 4)).expect("Failed to reshape!");
    return out_arr;
}





#[test]
fn test_similarity1() {
    let df: Array2<ColumnVal> = read_CSV_using_reader("Youtube-Spam-Dataset.csv");
    let (my_map, users) = map_users_to_words(&df); // my_map maps a user to a hashset of words they used. users is a vector of unique users in some random order
    
    let res = find_similarities(format!("Сергей Андреевич"), format!("Ed Garcon"), &my_map);
    let real_answer = 0.0;

    assert_eq!(res.unwrap(), real_answer);
}

#[test]
fn test_similarity2() {
    let df: Array2<ColumnVal> = read_CSV_using_reader("Youtube-Spam-Dataset.csv");
    let (my_map, users) = map_users_to_words(&df); // my_map maps a user to a hashset of words they used. users is a vector of unique users in some random order
    
    let res = find_similarities(format!("MrCurr3ncY"), format!("Julius NM"), &my_map);
    let real_answer: f64 = 3.0 / 11.0;

    assert_eq!(res.unwrap(), real_answer);
}

#[test]
fn test_graph_creation() {
    let df: Array2<ColumnVal> = read_CSV_using_reader("Youtube-Spam-Dataset.csv");
    let (my_map, users) = map_users_to_words(&df); // my_map maps a user to a hashset of words they used. users is a vector of unique users in some random order
    
    let users_shortened: Vec<String> = vec![format!("Sara"), format!("John"), format!("Teah")];
    let mut my_map_shortened: HashMap<String, HashSet<String>> = HashMap::new();

    let mut set1 = HashSet::new();
    let mut set2 = HashSet::new();
    let mut set3 = HashSet::new();

    set1.insert(format!("apple"));
    set1.insert(format!("banana"));

    set2.insert(format!("apple"));
    set2.insert(format!("banana"));

    set3.insert(format!("apple"));
    set3.insert(format!("cabbage"));

    my_map_shortened.insert(format!("Sara"), set1);
    my_map_shortened.insert(format!("John"), set2);
    my_map_shortened.insert(format!("Teah"), set3);

    let my_graph_shortened = create_graph(&users_shortened, &my_map_shortened, 1.0);
    
    let mut tester1 = false;
    let mut tester2 = false;
    let mut tester3 = false;
    let mut final_tester = false;

    if my_graph_shortened.contains_key(&format!("Sara")) && my_graph_shortened.contains_key(&format!("John")) && my_graph_shortened.contains_key(&format!("Teah")) {
        tester1 = true;
    }
    if my_graph_shortened.get(&format!("Sara")).unwrap().contains(&format!("John")) && my_graph_shortened.get(&format!("John")).unwrap().contains(&format!("Sara")) {
        tester2 = true;
    }
    if *my_graph_shortened.get(&format!("Teah")).unwrap() == Vec::<String>::new() {
        tester3 = true;
    }

    if tester1 && tester2 && tester3{
        final_tester = true;
    }
    assert_eq!(true, final_tester);

    
}

#[test]
fn test_num_graphs() {
    let df: Array2<ColumnVal> = read_CSV_using_reader("Youtube-Spam-Dataset.csv");
    let (my_map, users) = map_users_to_words(&df); // my_map maps a user to a hashset of words they used. users is a vector of unique users in some random order
    let my_graph_sim: HashMap<String, Vec<String>> = create_graph(&users, &my_map, 0.0); // maps a name to a vector of names of people whose similarity index with them is at least the threshold
    
    let num_graphs = find_num_disconnected_graphs(&my_graph_sim);
    println!("Num graphs: {:?}", num_graphs);
    assert_eq!(1, num_graphs); // since threshhold in this test is 0.0, every node should be connected to form one graph
}

#[test]
fn test_spam_finding() {
    let giant_vec: Vec<ColumnVal> = vec![ColumnVal::One("Sara".to_string()), ColumnVal::One("Pay me".to_string()), ColumnVal::One("Video1".to_string()), ColumnVal::Two(true),
                                        ColumnVal::One("John".to_string()), ColumnVal::One("Pay me".to_string()), ColumnVal::One("Video1".to_string()), ColumnVal::Two(true),
                                        ColumnVal::One("Teah".to_string()), ColumnVal::One("I love this video".to_string()), ColumnVal::One("Video1".to_string()), ColumnVal::Two(false),
                                        ColumnVal::One("Jei".to_string()), ColumnVal::One("Awesome".to_string()), ColumnVal::One("Video1".to_string()), ColumnVal::Two(false),
                                        ColumnVal::One("Maya".to_string()), ColumnVal::One("Subscribe to me".to_string()), ColumnVal::One("Video1".to_string()), ColumnVal::Two(true),
                                        ColumnVal::One("Sara".to_string()), ColumnVal::One("This is cool".to_string()), ColumnVal::One("Video2".to_string()), ColumnVal::Two(false),
                                        ColumnVal::One("Veri".to_string()), ColumnVal::One("Wow!".to_string()), ColumnVal::One("Video2".to_string()), ColumnVal::Two(false),
                                        ColumnVal::One("Veri".to_string()), ColumnVal::One("Pay me".to_string()), ColumnVal::One("Video3".to_string()), ColumnVal::Two(true)];
    
    let my_arr: Array2<ColumnVal> = Array::from_vec(giant_vec).into_shape((8,4)).expect("Unable to reshape");
    let (my_map, users) = map_users_to_words(&my_arr); // my_map maps a user to a hashset of words they used. users is a vector of unique users in some random order
    let (num_spam_users_mini, spammers_mini) = spam_functions::find_spam(&my_arr, &users);
    
    let mut tester1 = false;
    let mut tester2 = false;
    let mut final_tester = false;
    if num_spam_users_mini == 4 {
        tester1 = true;
    }
    if spammers_mini == vec!["Sara".to_string(), "John".to_string(), "Maya".to_string(), "Veri".to_string()] {
        tester2 = true;
    }

    if tester1 && tester2 {
        final_tester = true;
    }
    assert_eq!(true, final_tester);
}

#[test]
fn test_spam_arr_making() {
    let spam_arr: Array2<ColumnVal> = spam_functions::spam_specific_arr("Youtube-Spam-Dataset.csv");
    let mut unique_spammers: HashSet<String> = HashSet::new();
    for spammer in spam_arr.column(0) {
        match spammer {
            ColumnVal::One(name) => {
                unique_spammers.insert(name.to_string());
                ()
            },
            ColumnVal::Two(_) => (), // should not execute
        }
    }
    assert_eq!(871, unique_spammers.len());
}
