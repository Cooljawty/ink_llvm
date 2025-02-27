#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include "wrapper.h"

void print_status(Story story)
{
	printf("Status:%s\nContinue?: %s\nChoice count: %i\n", 
			story == NULL ? "Ended" : "Unfinished", 
			CanContinue(story) ? "True":"False"	,
			ChoiceCount(story)
	);
}

void string_test()
{
	string* test_str = new_string();
	write_string(test_str, "Hello!");
	char* read_buf = malloc(1);
	read_buf[0] = '\0';
	read_string(test_str, read_buf);
	printf("Test string: \'%s\'\n", read_buf);
	free(test_str->buffer);
	free(test_str);
}

int main() 
{
	//string_test();

	printf("Initilizing:\n");
	void* story = ContinueMaximally(NULL); 
	do{
		print_status(story);

		printf("Stepping..\n");
		while(CanContinue(story))
		{
			story = ContinueMaximally(story);
			print_status(story);
		}

		unsigned int choice;
		printf("Chose choice: ");
		scanf("%u", &choice);
		ChooseChoiceIndex(story, choice);
		print_status(story);
		
		story = ContinueMaximally(story); 
	} while(CanContinue(story) || ChoiceCount(story) > 0);

	return 0;
}
