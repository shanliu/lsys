<?php

/** @generate-function-entries */

namespace LsExt {
    class Exception extends \Exception
    {
    }
    class LibArea
    {
        /**
         * @throw Exception
         */
        static public function initCsv(
            string $code_path,
            string $geo_path,
            string $index_path = "",
            int $index_size = 0,
            bool $gz = true
        ): void {
        }
        /**
         * @throw Exception
         */
        static public function initSqlite(
            string $sqlite_sql,
            string $index_path = "",
            int $index_size = 0
        ): void {
        }
        /**
         * @throw Exception
         */
        static public function initMysql(
            string $uri,
            string $index_path = "",
            int $index_size = 0
        ): void {
        }
        /**
         * @throw Exception
         */
        static public function shutdown(): void
        {
        }
        /**
         * @throw Exception
         */
        static public function geoReload(): void
        {
        }

        /**
         * @throw Exception
         */
        static public function codeReload(): void
        {
        }
        /**
         * @throw Exception
         */
        static public function codeChilds(string $code): array
        {
            return [];
        }
        /**
         * @throw Exception
         */
        static public function codeSearch(string $code, int $limit = 10): array
        {
            return [];
        }
        /**
         * @throw Exception
         */
        static public function codeFind(string $code): array
        {
            return [];
        }
        /**
         * @throw Exception
         */
        static public function codeRelated(string $code): array
        {
            return [];
        }
        /**
         * @throw Exception
         */
        static public function geoSearch(float $lat, float $lng): array
        {
            return [];
        }
    }
}
