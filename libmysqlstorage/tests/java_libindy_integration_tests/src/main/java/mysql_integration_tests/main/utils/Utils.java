package mysql_integration_tests.main.utils;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;

import java.io.IOException;
import java.util.HashMap;
import java.util.Random;
import java.util.SplittableRandom;

public class Utils {
    private static final String CHARS = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890";

    public static int[] range(int index){
        int[] arr = new int[index];
        for(int i=1;i<index;i++){
            arr[i]=i;
        }
        return arr;
    }

    public static HashMap<String, String> getHashMapFromJsonString(String jsonString){
        HashMap<String, String> map = new HashMap<>();
        ObjectMapper mapperObj = new ObjectMapper();
        try {
            map = mapperObj.readValue(jsonString, new TypeReference<HashMap<String, String>>(){});
        } catch (IOException e) {
            e.printStackTrace();
        }
        return map;
    }

    public static String getJsonStringFromHashMap(HashMap<String, String> map){
        ObjectMapper mapperObj = new ObjectMapper();
        String jsonString = null;
        try {
            jsonString = mapperObj.writeValueAsString(map);
        } catch (JsonProcessingException e) {
            e.printStackTrace();
        }
        return jsonString;
    }

    public static String generateRandomString(int size){
        StringBuffer randStr = new StringBuffer();
        for(int i=0; i<size; i++){
            int number = new SplittableRandom().nextInt(0, CHARS.length());
            char ch = CHARS.charAt(number);
            randStr.append(ch);
        }
        return randStr.toString();
    }

    public static String generateRandomRecordValue(){
        return  new SplittableRandom().ints(300, 0, 255).parallel().toArray().toString();
    }

}
