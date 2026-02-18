// Web Worker 用于计算文件 MD5，避免阻塞主线程
import SparkMD5 from 'spark-md5';

interface Message {
  type: 'calculate' | 'cancel';
  file?: Blob;
  chunkSize?: number;
}

interface ProgressMessage {
  type: 'progress';
  progress: number;
}

interface ResultMessage {
  type: 'result';
  md5: string;
}

interface ErrorMessage {
  type: 'error';
  error: string;
}

let isCancelled = false;

self.onmessage = async (event: MessageEvent<Message>) => {
  const { type, file, chunkSize = 2 * 1024 * 1024 } = event.data;

  if (type === 'cancel') {
    isCancelled = true;
    return;
  }

  if (type === 'calculate' && file) {
    try {
      isCancelled = false;
      const md5 = await calculateMD5(file, chunkSize);
      const result: ResultMessage = { type: 'result', md5 };
      self.postMessage(result);
    } catch (error) {
      const errorMsg: ErrorMessage = {
        type: 'error',
        error: error instanceof Error ? error.message : '未知错误',
      };
      self.postMessage(errorMsg);
    }
  }
};

async function calculateMD5(file: Blob, chunkSize: number): Promise<string> {
  const chunks = Math.ceil(file.size / chunkSize);
  let currentChunk = 0;
  const spark = new SparkMD5.ArrayBuffer();

  return new Promise((resolve, reject) => {
    const reader = new FileReader();

    reader.onload = (e) => {
      if (isCancelled) {
        reject(new Error('计算已取消'));
        return;
      }

      if (e.target?.result) {
        spark.append(e.target.result as ArrayBuffer);
      }

      currentChunk++;
      const progress = Math.round((currentChunk / chunks) * 100);
      const progressMsg: ProgressMessage = { type: 'progress', progress };
      self.postMessage(progressMsg);

      if (currentChunk < chunks) {
        loadNext();
      } else {
        resolve(spark.end());
      }
    };

    reader.onerror = () => {
      reject(new Error('文件读取失败'));
    };

    const loadNext = () => {
      const start = currentChunk * chunkSize;
      const end = Math.min(start + chunkSize, file.size);
      reader.readAsArrayBuffer(file.slice(start, end));
    };

    loadNext();
  });
}
