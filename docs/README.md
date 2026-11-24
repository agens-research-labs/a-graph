# Documentation

- [The architecture](architecture.md): the entities, the components and the managers, and how a file becomes a program.
- [Agents and software](agents-and-software.md): the agents in the file, the software in the program, and what observability lets you measure.
- [The command line](cli.md): how to run a graph, the result, the errors, the exit codes.
- [Examples](examples.md): the three graphs of `examples/`, what each shows and how to run it.
- [The graph file](file.md): the blocks, the ids, the secrets, the allowed hosts.
- [The reference](graph.md): every key of the graph file, generated from the code.
- [The HTTP service](serve.md): how to run graphs over HTTP, the sessions, the rules for a graph from outside.
- [The service file](service.md): every key of the file that starts the HTTP service, generated from the code.

The blocks of the graph file:

- [Models](models.md): the providers, their credentials and what each can do, batch, reasoning.
- [Prompts](prompts.md): the placeholders and their rules.
- [Nodes](nodes.md): what a node receives and produces, and the validation nodes.
- [Edges](edges.md): the order of the nodes, the groups, `react`.
- [Tools](tools.md): the contract of a tool and the loop of the calls.
- [MCP servers](mcp.md): how the servers are connected and called.
- [Skills](skills.md): how the model uses a skill, and its scripts.
- [Media](media.md): the media a node reads and produces.
- [Utilities](utilities.md): the programs that change the data of a node.
- [Embeddings](embeddings.md): the models that turn a text into a vector.
- [Vector databases](vectordb.md): the databases a node searches, and `{{chunks}}`.
- [Memory](memory.md): the file of a memory, what the model gets, the summary.
- [Blackboards](blackboards.md): the shared notes and their header.
- [Observability](observability.md): the log, the trace, the display.

To work on the code, see [CONTRIBUTING](../CONTRIBUTING.md).
