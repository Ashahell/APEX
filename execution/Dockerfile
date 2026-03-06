FROM python:3.11-slim

WORKDIR /app

RUN apt-get update && apt-get install -y \
    curl \
    git \
    && rm -rf /var/lib/apt/lists/*

RUN pip install --no-cache-dir poetry

COPY pyproject.toml poetry.lock* ./

RUN poetry config virtualenvs.create false && poetry install --no-interaction --no-root

COPY src/ ./src/

ENV PYTHONPATH=/app

CMD ["python", "-u", "-c", "import sys; print('APEX Execution Container Ready'); import time; time.sleep(365*24*60)"]
