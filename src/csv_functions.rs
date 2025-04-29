use ndarray::{Array, Array2};
use csv::ReaderBuilder;







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
