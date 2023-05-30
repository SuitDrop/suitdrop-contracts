# Start from a Rust base image
FROM rust:1.69.0

# Set the current working directory in the container
WORKDIR /usr/src/app

# Copy the entire project to the working directory
COPY . .

# Build the project, including tests
RUN cargo build --verbose --all-targets

# The command to run when the container starts
CMD ["cargo", "all-test"]
