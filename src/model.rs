use tch::Cuda;

pub fn cuda_available() -> bool {
    Cuda::is_available()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gpu_available() {
        assert!(cuda_available());
    }
}
