# Use an official Python image
FROM python:3.10

# Set the working directory
WORKDIR /code

# Copy requirements and install dependencies (if applicable)
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy the application code
COPY . .

# Expose the port
EXPOSE 8000

# Run FastAPI
CMD ["uvicorn", "main:app", "--host", "0.0.0.0", "--port", "8000"]
