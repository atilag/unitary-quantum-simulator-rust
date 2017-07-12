#!/bin/bash
# TODO Generalize
# TODO gdb seems to have problems with Python virtual envs like Anaconda, so
# debugging with gdb is not working so far
export PYTHONHOME=~/anaconda3:~/anaconda3/envs/TPIR
export PYTHONPATH=$PYTHONPATH:~/anaconda3/envs/TPIR:~/ibm/quantum/private-qiskit-sdk-py-dev
export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:~/anaconda3/lib
export RUST_BACKTRACE=1

command="cargo test circuit1 -- --nocapture"

case $1 in
    --help|help)
        echo "Usage: $0 [command]"
	    echo "Command:"
	    echo "  build			   Runs cargo just to build"
	    echo "  test debug		   Runs cargo test with RUST_LOG=debug"
	    echo "  test info          Runs cargo test with RUST_LOG=info"
	    echo "  debug /path/to/bin Runs the debugger (gdb)"
	    echo ""
	    exit 1
	    ;;
    build)
        command="cargo build"
	    ;;
    test)
    	case $2 in
    	    debug)
                export RUST_LOG=unitary_simulator=debug
    		    ;;
    	    info)
    	       export RUST_LOG=unitary_simulator=info
    		   ;;
            *)
               export RUST_LOG=unitary_simulator=debug
               ;;
    	esac
    	;;
    debug)
    	if [ -z "$2" ]
    	then
    	    echo "Error: No path to binary supplied"
    	    exit 1
    	fi
    	command="gdb $2"
    	;;
    *)
        export RUST_LOG=unitary_simulator=debug
        ;;
esac

$command
echo "Command executed: "
echo $command RUST_LOG=$RUST_LOG RUST_BACKTRACE=$RUST_BACKTRACE
