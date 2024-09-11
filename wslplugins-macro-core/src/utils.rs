#[macro_export]
macro_rules! acc_syn_result {
    ($($result:expr),+ $(,)?) => {{
        let mut err: Option<::syn::Error> = None;
        $(
            match $result.as_ref() {
                Ok(_) => {},
                Err(new_err) => {
                    if let Some(ref mut unwrap_err) = err {
                        unwrap_err.combine(new_err.clone());
                    } else {
                        err = Some(new_err.clone());
                    }
                }
            }
        )+

        if let Some(err) = err {
            Err(err)
        } else {
            Ok((
                $(
                    $result.unwrap(),
                )+
            ))
        }
    }};
}

#[cfg(test)]
mod tests {
    use syn::{Error, Result};

    #[test]
    fn test_all_ok_results() {
        let result1: Result<i32> = Ok(10);
        let result2: Result<String> = Ok("Success".to_string());
        let result3: Result<f64> = Ok(3.14);

        let combined_result = acc_syn_result!(result1, result2, result3);
        match combined_result {
            Ok((val1, val2, val3)) => {
                assert_eq!(val1, 10);
                assert_eq!(val2, "Success".to_string());
                assert_eq!(val3, 3.14);
            }
            Err(_) => panic!("Expected all Ok results"),
        }
    }

    #[test]
    fn test_single_error() {
        let result1: Result<i32> = Ok(10);
        let result2: Result<String> = Err(Error::new_spanned(&"dummy", "Error in result2"));
        let result3: Result<f64> = Ok(3.14);

        let combined_result = acc_syn_result!(result1, result2, result3);
        match combined_result {
            Ok(_) => panic!("Expected an error, but got Ok"),
            Err(e) => assert_eq!(e.to_string(), "Error in result2"),
        }
    }

    #[test]
    fn test_multiple_errors() {
        let result1: Result<i32> = Err(Error::new_spanned(&"dummy1", "Error in result1"));
        let result2: Result<String> = Err(Error::new_spanned(&"dummy2", "Error in result2"));
        let result3: Result<f64> = Ok(3.14);

        let combined_result = acc_syn_result!(result1, result2, result3);
        match combined_result {
            Ok(_) => panic!("Expected an error, but got Ok"),
            Err(e) => {
                let tokens = e.into_compile_error();
                let token_string = tokens.to_string();
                assert!(token_string.contains("Error in result1"));
                assert!(token_string.contains("Error in result2"));
            }
        }
    }

    #[test]
    fn test_ok_and_error() {
        let result1: Result<i32> = Ok(10);
        let result2: Result<String> = Err(Error::new_spanned(&"dummy", "Error in result2"));

        let combined_result = acc_syn_result!(result1, result2);
        match combined_result {
            Ok(_) => panic!("Expected an error, but got Ok"),
            Err(e) => assert_eq!(e.to_string(), "Error in result2"),
        }
    }
}
