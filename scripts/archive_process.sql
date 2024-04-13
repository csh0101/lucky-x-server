CREATE TABLE `archive_process` (
  `id` BIGINT(20) NOT NULL AUTO_INCREMENT,
  `status` ENUM('Pending','Finished','Failed')
  `target_dir` VARCHAR(255) NOT NULL,
  `zip_filename` VARCHAR(255) NOT NULL,
  `oss_url` VARCHAR(255) ,
  `start_time` DATETIME(6) DEFAULT NULL,
  `end_time` DATETIME(6) DEFAULT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;