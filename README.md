# Unitary simulator
Rust version of the Unitary Simulator found in the Python QISKit.

## General
Includes a run.sh script that will handle all the required setup to run
the simulator tests, and build the libraries.

## Prerequisites
It requires Rust building environment (stable branch). Follow [this guide](https://www.rust-lang.org/en-US/install.html)
to install it, and Anaconda, as [specified](https://github.com/IBM/qiskit-sdk-py/blob/master/README.md) by the Python Qiskit SDK.

Python QISKit Core is going to be migrated to Rust progressively, in the
meantime we still need to interface with some of the Python components in Rust,
so a script called run.sh is provided to abstract the user from some of the
complex low-level details required to run Qiskit Python SDK into our Rust
environment.
Please, edit run.sh script and change the paths according to your Anaconda
installation.

## Building
Build the Development version with debugging information (default):
> ./run.sh build dev

If you want to build the Release version (optimized, no debugging info):
> ./run.sh build rel


## Running tests
To run the tests we have various options:
1. Run specific test
> ./run.sh test name_of_test

2. Run all tests
> ./run.sh test

All tests will run with no logs. If you want to run the tests with logs enabled
you can run:
> ./run.sh test <test_name> <debug|info>

Example:
> ./run.sh test circuit1 debug


## Authors (alphabetical)
Juan Gómez Mosquera


## License
Apache 2 license.
