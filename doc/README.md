# Documentation

This directory contains the project documentation built with MkDocs.

## Running Locally

### Using Python Script (Recommended)

Run the provided Python script to automatically set up and serve the documentation:

```bash
python start_docs.py
```

This will:
- Create a virtual environment if needed
- Install required dependencies
- Start the MkDocs development server
- Open your browser to http://localhost:8000

### Manual Setup

If you prefer to set up manually:

1. Create a virtual environment:
   ```bash
   python -m venv venv
   ```

2. Activate the virtual environment:
   - Linux/Mac: `source venv/bin/activate`
   - Windows: `venv\Scripts\activate`

3. Install dependencies:
   ```bash
   pip install -r docs/requirements.txt
   ```

4. Serve the documentation:
   ```bash
   mkdocs serve
   ```

5. Open your browser to http://localhost:8000

## Building

To build the static site:

```bash
mkdocs build
```

The built site will be in the `site/` directory.
