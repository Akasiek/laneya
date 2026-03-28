## [1.0.1] - 2026-03-28

### 🐛 Bug Fixes

- Specify Dockerfile path in workflows for build and publish tasks

### 🚜 Refactor

- Reorganize Docker setup with improved structure and user permissions

### 📚 Documentation

- Fix casing in Docker image reference in README
## [1.0.0] - 2026-03-28

### 🚀 Features

- Add MIT License to the project
- Implement channel management features
- Add configuration for filtering YouTube Shorts
- Implement video pagination and enhance video display
- Update published date formatting in video repository
- Update channel handling to fetch channel name dynamically when creating
- Add database migration initialization
- Use HOST environment variable for web server binding fallback
- Add Docker configuration for application deployment
- Add feed refresh interval configuration
- Add Forgejo Actions workflow for Docker image build and publish
- Add timezone configuration for video timestamps
- Add bulk import functionality for YouTube subscriptions
- Add unique index for channel_id in channels table
- Improve bulk import functionality and add feedback messaging
- Implement request throttling for YouTube feed fetching
- Add manual feed refresh functionality
- Enhance Docker configuration for improved security and user management
- Add security scan workflow using Trivy
- Update Trivy action
- Update Trivy action to version 0.35.0
- Improve navigation styling and z-index management
- Replace Trivy with Docker Scout for CVE scanning
- Update CodeQL SARIF upload action to version 4

### 🐛 Bug Fixes

- Update foreign key constraints for channel_id

### 💼 Other

- Initialize project structure and add core functionality

### 🚜 Refactor

- Move reusable templates to components directory
- Separate channel handlers into API and page-specific modules
- Update video handling by introducing VideoResource and modularizing templates
- Rename `filter_shorts` to `filter_out_shorts` for clarity
- Rename `init_tracer` and `init_web_server` to `init` for consistency
- Rename `per_page` to `videos_per_page` for clarity
- Restructure channel components into modular templates
- Update nav component for improved accessibility and styling
- Order channels by name in find_all method
- Improve table row formatting and add h-full utility class
- Enhance button alignment and add vertical alignment utility
- Rename and restructure feed refresh job handling
- Update form actions to use API endpoints
- Simplify WebSocket handler and improve initial video grid sending
- Reformat channel insertion mapping logic in repository
- Remove unused nav_channels block from channels template
- Introduce `html_error` macro and replace repetitive error rendering logic
- Update `fetch_channel_name` to return full feed data
- Channel API Handler and associated files
- Update channel creation to return Channel object instead of ID
- Improve error logging in channel feed fetching
- Simplify Arc usage for reqwest client in channel repository
- Enhance error handling for channel feed fetching
- Streamline video upsert logic and enhance error handling
- Update format_published_at function parameter name for clarity
- Rename channel_db_id parameter to channel_id for clarity
- Simplify video pagination logic and enhance readability
- Rename fetch_channel_feed_data to read_channel_feed for clarity
- Simplify environment variable parsing in Config
- Update send_refresh_notification to use broadcast channel

### 📚 Documentation

- Update README.md with enhanced features and setup instructions
- Update README.md for Docker deployment instructions
- Clarify grammar in Quenya definition in README.md
- Add some docinfo about bulk_create function in channel_repository
- Add comment about deleting Videos before delete Channel
- Add AGENTS.md
- Update README with new screenshot and formatting improvements

### 🎨 Styling

- Update title formatting in HTML files
- Enhance video card layout with flexbox and improve structure
- Remove unnecessary class from video card title for cleaner markup

### ⚙️ Miscellaneous Tasks

- Update tailwind.css
- Update tailwind.css
- Update .gitignore to include .env file
- Add Docker publish workflow
- Update tailwind.css
- Update tailwind.css
- Remove Forgejo Docker publish GitHub Action
- Remove branch specification from Docker publish workflow
- Update tailwind.css
- Update Dockerfile to improve package installation
