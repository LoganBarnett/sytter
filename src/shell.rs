use crate::error::AppError;

pub fn shell_exec_check(
    script: &String,
    expected_exit_codes: &Vec<usize>,
) -> Result<bool, AppError> {
    Ok(true)
}
