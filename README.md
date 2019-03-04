# enya

enya builds on [railcar](https://github.com/oracle/railcar), an OCI compliant container runtime, to form a Special Purpose Container Runtime.

Any general additions will be contributed upstream.

# Overview

![enya](Enya.png?raw=true "Architecture")

enya initializes pid 1 as the **System** process (similar to init process in railcar). 
The System process constructs the specified cgroups setup and takes a share (%) of CPU and Memory.
It then places the actual container **Process** (child) into a new cgroup, in order to have full control of the running container binary.

# Features

enya currently offers:

1.  Subscription service, where processes, local or non-local can subscribe to metric reports of the enya **Process**.

There is plans to add additional features. One example being more advanced traffic control (tc), 
where rules can be set during startup or even on the fly during runtime.

enya is suited for distributed runtimes, especially where dynamic scheduling is required.

# API

The [API](api/protobuf/messages.proto) is defined in Protobuf (version 3) and currently supports [kompact](https://github.com/kompics/kompact). However, it can be extended to gRPC as well.

# License

enya is licensed the under Apache License 2.0.

See [LICENSE](LICENSE) for more details.
