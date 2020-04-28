# REST API

The REST API can be used by starting the gateway with the [bayard-rest](cli/bayard-rest.md) CLI.  
Several APIs are available to manage Bayard over the HTTP.
See the following list:

- [Get document API](api/get.md)  
&nbsp;&nbsp;&nbsp;&nbsp; Get API gets a document with the specified ID.

- [Set document API](api/set.md)  
&nbsp;&nbsp;&nbsp;&nbsp; Set document API puts a document with the specified ID and field. If specify an existing ID, it will be overwritten with the new document.

- [Delete document API](api/delete.md)  
&nbsp;&nbsp;&nbsp;&nbsp; Delete document API deletes a document with the specified ID.

- [Bulk set documents API](api/bulk-set.md)  
&nbsp;&nbsp;&nbsp;&nbsp; Bulk set API sets documents in bulk with the specified ID and field. If specify an existing ID, it will be overwritten with the new document.

- [Bulk delete documents API](api/bulk-delete.md)  
&nbsp;&nbsp;&nbsp;&nbsp; Bulk delete documents API deletes documents in bulk with the specified ID.

- [Commit API](api/commit.md)  
&nbsp;&nbsp;&nbsp;&nbsp; Commit API commits updates made to the index.

- [Rollback API](api/rollback.md)  
&nbsp;&nbsp;&nbsp;&nbsp; Rollback API rolls back any updates made to the index to the last committed state.

- [Merge API](api/merge.md)  
&nbsp;&nbsp;&nbsp;&nbsp; Merge API merges fragmented segments in the index.

- [Schema API](api/schema.md)  
&nbsp;&nbsp;&nbsp;&nbsp; Schema API shows the index schema that the server applied.

- [Search API](api/search.md)  
&nbsp;&nbsp;&nbsp;&nbsp; Search API searches documents from the index.

- [Status API](api/status.md)  
&nbsp;&nbsp;&nbsp;&nbsp; Status API shows the cluster that the specified server is joining.

- [Metrics API](api/metrics.md)  
&nbsp;&nbsp;&nbsp;&nbsp; Metrics API shows the server metrics of the specified server. The metrics are output in Prometheus exposition format.
