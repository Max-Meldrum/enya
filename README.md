# enya

enya builds on [railcar](https://github.com/oracle/railcar), an OCI compliant container runtime, to form a Special Purpose Container Runtime.

Any general additions will be contributed upstream.

# Features

enya initializes pid 1 as the *System* process (similar to init process in railcar). 
The *System* process constructs the specified cgroups setup and takes a share (%) of CPU and Memory.
It then places the actual container *Process* (child) into a new cgroup, in order to have full control of the running container binary.

The *System* process offers a subscription service, where MetricReport's are sent out to each subscriber. The subscription service is not limited to
the local container *Process*, but also processes over the network. enya is suited for distributed runtimes, where dynamic profiling is important.

# API

The [API](api/protobuf/messages.proto) is defined in Protobuf (version 3) and currently supports [kompact](https://github.com/kompics/kompact). However, it can be extended to gRPC as well.

## License ##

enya is licensed the under Apache License 2.0.

See [LICENSE](LICENSE) for more details.
