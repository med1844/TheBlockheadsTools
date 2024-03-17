FROM --platform=linux/386 python:alpine3.19
WORKDIR /app
RUN apk add --no-cache patch gcc musl-dev linux-headers git
COPY requirements.txt /app
RUN pip install --no-cache-dir -r requirements.txt
# RUN pip install 'git+https://github.com/bretello/pdbpp@master'
# RUN pip install pytest
COPY . /app
# CMD ["python3", "-m", "pdb", "gameSave.py"]
