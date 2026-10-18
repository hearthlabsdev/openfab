# Server Library
This is intended to be a library for shared models between the WebUI and API servers but the refactor isn't complete.

## File Reconciliation
Planned Features:

1. Database Comparison
Shallow checks can be performed when listing files by checking path, size, modified to ensure record is up to date. If an unknown file is seen spawn a background worker to download the file, generate its hash, and update the file entry table. If a file is missing, delete the entry from device_files table.

2. Full File Integrety Check
There should be a means of running a full file integrity check between the [`DeviceFileORM`] and [`FileEntry`] as this would be an expensive operaiton, it is something that should be triggered manually. During this process the server would download each ifle on the printer in turn, and match it's hashes to ensure file integrety.

3. File Backup
As a manual action an admin can run a file backup that downloads all files, then reuploads them to the backend storage (such as S3, or FTPS Server). This could also be run using an API, or scheduled.