#!/bin/bash
# TODO gdb seems to have problems with Python virtual envs like Anaconda, so
# debugging with gdb is not working so far
export PYTHONHOME=~/anaconda3:~/anaconda3/envs/TPIR
export PYTHONPATH=$PYTHONPATH:~/anaconda3/envs/TPIR:~/ibm/quantum/private-qiskit-sdk-py-dev
export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:~/anaconda3/lib
export RUST_BACKTRACE=1

command="cargo test -- --nocapture"

case $1 in
    --help|help)
        echo "Usage: $0 [command]"
	    echo "Command:"
	    echo "  build [rel|dev]             Builds all the source code for"
        echo "                              release or development (debugging)"
        echo "  test [test]                 Runs the specified test"
	    echo "  test [test] [debug|info]    Runs the specified test with the"
        echo "                              specified log level: debug or info"
        echo "  *Experimental* "
	    echo "  debug </path/to/bin>        Runs the debugger (gdb)"
        echo ""
	    echo "If no params are specified, the default will launch all "
        echo "the tests with no logs."
        echo "RUST_BACKTRACE will be always set to 1"
	    exit 1
	    ;;
    build)
        command="cargo build"
	    ;;
    test)
        if [ -z "$2" ]
        then
            break
        fi

        command="cargo test $2 -- --nocapture"

    	case $3 in
            debug)
                environ="RUST_LOG=unitary_simulator=debug"
                ;;
    	    info)
                environ="RUST_LOG=unitary_simulator=info"
    		    ;;
    	esac
    	;;
    debug)
    	if [ -z "$2" ]
    	then
    	    echo "Error: No path to binary supplied!"
    	    exit 1
    	fi
    	command="gdb $2"
    	;;
esac

if [ -n "$environ" ]
then
    export $environ
fi

$command
echo "Command executed: "
echo RUST_LOG=$RUST_LOG RUST_BACKTRACE=$RUST_BACKTRACE $command
