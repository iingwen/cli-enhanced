
# v0.26.0
## Breaking

- The `create bucket` flag `--transform-tag` is now removed.

# v0.25.0
- Fixes issue when getting streams that have multiple filters on single user property
- Fixes issue where upper case file names would not be matched in `parse` 
- Reduce batch size when deleting comment batches
- Support attachment type filters
- support getting stats for `get buckets` 
- Show usage on `get quotas`

# v0.24.0
- BREAKING: the `--context` option is now required. Users need to opt
  out if they don't want to provide this for every command
- BREAKING: the `--context` option is always a required field for internal users

# v0.23.0

- Add `get emails`
- Added support for `--auto-increase-up-to` when creating quotas.
- Support spans format for entities 

# v0.22.2

- Fix a bug where some label annotations cannot be applied

# v0.22.1

- minor api improvements

# v0.22.0

- Add integration commands

# v0.21.5

- Fix a bug where stream responses were not correctly parsed
- Fix a bug where streams were not correctly advanced

# v0.21.4

- Add messages filters
- Fixes `required` field error when interacting with datasets

## v0.21.3

- Reduce batch size for parse emls

## v0.21.2

- Add get audit events command
- Add ability to parse .emls

## v0.21.1

- Add more stream stats

## v0.21.0

- Fix url used for fetching streams
- Return `is_end_sequence` on stream fetch
- Make `transform_tag` optional on `create bucket`
- Retry `put_emails` requests
- Add `get stream-stats` to expose and compare model validation

## v0.20.0

- Add ability to get dataset stats
- Show Global Permissions in `get users`
- Upgrade `ordered-float` version, which is exposed in the public crate api.
- Add ability to filter users by project and permission
- Add feature to parse unicode msg files

## v0.19.0

- Add create streams command
- Show source statistics in table when getting sources

## v0.18.2

- Add ability to filter on user properties when getting comments

## v0.18.1

- Add comment id to document object in api

## v0.18.0

- Add label filter when downloading comments with predictions
- Retry requests on request error

## v0.17.2

- Retry TOO_MANY_REQUESTS

## v0.17.1

- Support markup in signatures
- Fix bug where annotations may have been uploaded before comments, causing a failure

## v0.17.0

- Always retry on connection issues
- Upload annotations in parallel

## v0.16.1

- Add attachments to `sync-raw-email`

## v0.16.0

- Add command to list quotas for current tenant
- Show correct statistics when downloading comments
- Add `sync-raw-emails` to api

## v0.15.0

- Add support for markup on comments

## v0.14.0

- Add a warning for UiPath cloud users when an operation will charge ai units

## v0.13.4

- Add user property filters to the query api

## v0.13.3

- Add recent as an option for the query api

## v0.13.2

- Skip serialization of continuation on `None`

## v0.13.1

- Add `no-charge` flag to create comment/email commands
- Add comment and label filters to `get_statistics`
- Add timeseries to `get_statistics`
- Add `query_dataset` to api

## Added

- `re get comments` returns label properties

# v0.12.3

## Added

- `re create quota` to set a quota in a tenant

# v0.12.2

- Rename "triggers" to "streams" following the rename in the API
- Removed semantic url joins to support deployments within a subdirectory
- Added functionality to use moon forms both in `LabelDef`s and in `AnnotatedComments`s

## Added

- `re get comments` will now return auto-thresholds for predicted labels if provided with a `--model-version` parameter
- `re update users` for bulk user permission updates
- Option to send welcome email on create user

# v0.12.1

## Added

- `re update source` can now update the source's transform tag
- `re get source` and `re get sources` will show bucket name if exists.
- `re get comments` can now download predictions within a given timerange

# v0.12.0

## Added

- Display project ids when listing projects
- Add support for getting or deleting a single user
- Upgrade all dependencies to their latest released version
- Enable retry logic for uploading annotations
- Add support for optionally setting a transform tag on a source

# v0.11.0

## Breaking

- Renames organisation -> project throughout, including in the CLI command line arguments for consistency with the new API
- `re create dataset` will default to sentiment disabled if `--has-sentiment` is not provided.