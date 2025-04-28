use ndarray::{Array, Array2};
use csv::ReaderBuilder;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;

// Module summary: This module contains spam-specific functions--the functions that will be used to build
// the graph consisting of only spammers' nodes.



// purpose: a duplicate of the read_CSV_using_reader() function, except it filters out the rows that correspond to non-spam content.
// input: a &str that is the name of the CSV to load in
// output: an Array2 of the data that correspond to rows (comments and their data) classified as spam
// iterates over rdr.records() and matches each to either a valid line or an error reading the line. Takes the columns of interest: 1, 3, 4, 5
pub fn spam_specific_arr(path: &str) -> Array2<crate::ColumnVal> {
    let mut rdr = csv::ReaderBuilder::new()
    .has_headers(true)
    .delimiter(b',')
    .double_quote(true)
    .escape(Some(b'\\'))
    .flexible(false)
    .from_path(path).unwrap();


    let headers = rdr.headers();

    let mut counter = 0; // this counts number of rows

    let mut giant_vec: Vec<crate::ColumnVal> = Vec::new(); // later, we'll form the array from this giant vector and reshape it to have the correct dimensions

    for result in rdr.records() {
        match result {
            Ok(record) => { // record is a line
                for (num, item) in record.iter().enumerate() { // item is a particular cell
                    if num == 1 || num == 3 || num == 4 { // this is user, comment content, video name cols
                        giant_vec.push(crate::ColumnVal::One(item.to_string()));
                    }
                    if num == 5 { // this is classification col
                        if item == "1" {
                            counter += 1;
                            giant_vec.push(crate::ColumnVal::Two(true));
                        } else if item == "0" {
                            // remove those three your just pushed in, since this person is not a spammer
                            giant_vec.pop();
                            giant_vec.pop();
                            giant_vec.pop();
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
    
    let out_arr: Array2<crate::ColumnVal> = Array::from_vec(giant_vec).into_shape((counter, 4)).expect("Failed to reshape!");
    return out_arr;
}




// purpose: find information about the data specifically marked as spam
// input: the array of data and a vector of unique users
// output: the number of unique spammers and a vector of those unique spam users' names
// it iterates over the column of the array that corresponds to the classification as spam or not
pub fn find_spam(arr_data: &Array2<crate::ColumnVal>, unique_users: &Vec<String>) -> (u32, Vec<String>) { // returns number of spam users and a vector of spam users
    // We know the spam column is in the index col 3 of arr_data, a col of bools
    // A user will be considered a spammer if at least ONE of their comments is marked as spam

    let mut seen_before: Vec<String> = Vec::new();
    let mut spam_users: Vec<String> = Vec::new();
    let mut counter: u32 = 0;
    for (num, item) in arr_data.column(3).iter().enumerate() { // col 3 is the classification col
        let my_bool: bool = match item { // unpack the ColumnVal with a match statement
            crate::ColumnVal::One(_) => false,
            crate::ColumnVal::Two(some_bool) => *some_bool,
        };
        let given_name: String = match &arr_data[[num, 0]] { // similar unpacking here
            crate::ColumnVal::One(name) => name.to_string(),
            crate::ColumnVal::Two(some_bool) => format!("Dummy name"), // should not execute
        };
        if my_bool && seen_before.contains(&given_name) == false { // then we have found a new unique user
            counter += 1;
            spam_users.push(given_name.to_string());
            seen_before.push(given_name.to_string());
        }
    }
    return (counter, spam_users);
}



// purpose: find the best spammer, or the spammers that tied for best spammer. The best spammer is determined by having the max number of neighbors in the graph.
// input: a graph mapping users to their vector of neighbors
// output: a hashset of users' names or a single name for the best spammers(s)
// iterates over the graph's keys and counts the neighbors for each user to find the user(s) with the most neighbors
pub fn find_best_spammer(graph: &HashMap<String, Vec<String>>) -> HashSet<String> {
    let mut max_neighbors = 0;
    let mut best_spammers: HashSet<String> = HashSet::new();
    for spammer in graph.keys() {
        if graph.get(&spammer.clone()).unwrap().len() > max_neighbors { // we should clear the previous best spammer and set this one to be the best
            best_spammers.clear();
            max_neighbors = graph.get(&spammer.clone()).unwrap().len();
            best_spammers.insert(spammer.to_string());
        } else if graph.get(&spammer.clone()).unwrap().len() == max_neighbors { // we have found someone who tied for best spammer
            best_spammers.insert(spammer.to_string());
        } else {
            ;
        }
    }

    return best_spammers;
}
