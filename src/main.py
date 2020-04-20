import tornado.ioloop
import tornado.web
import json
import time

class TerraformHandler(tornado.web.RequestHandler):
    logs = 0
    metrics = 0
    def get(self):
      return self.do()

    def post(self):
      return self.do()

    def put(self):
      return self.do()

    def do(self):
        self.write(json.dumps(
          {
            "source": {
              "url": "http://sumock.sumock:3000/receiver"
            }
          }
        ))


class ReceiverHandler(tornado.web.RequestHandler):
    logs = 0
    metrics = 0
    last_timestamp = time.time()

    def get(self):
      return self.do()

    def post(self):
      return self.do()

    def do(self):
        if self.request.headers['content-type'] == "application/vnd.sumologic.prometheus":
            ReceiverHandler.metrics += len(self.request.body.decode().strip().split("\n"))
        elif self.request.headers['content-type'] == "application/x-www-form-urlencoded":
            ReceiverHandler.logs += len(self.request.body.decode().strip().split("\n"))
        if time.time() - self.last_timestamp > 60:
            print(self.last_timestamp, "logs: {}".format(self.logs), "metrics: {}".format(self.metrics))
            ReceiverHandler.metrics = 0
            ReceiverHandler.logs = 0
            ReceiverHandler.last_timestamp = time.time()
        self.write("")

def make_app():
    return tornado.web.Application([
        (r"/terraform.*", TerraformHandler),
        (r"/.*", ReceiverHandler),
    ])

if __name__ == "__main__":
    app = make_app()
    app.listen(3000)
    tornado.ioloop.IOLoop.current().start()